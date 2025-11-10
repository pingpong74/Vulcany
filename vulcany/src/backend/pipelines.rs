use ash::vk;

use crate::backend::device::InnerDevice;

use serde::{Deserialize, Serialize};

use crate::{ComputePipelineDescription, RasterizationPipelineDescription};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ShaderCacheEntry {
    slang: String,
    spv: String,
    timestamp: u64,
}

pub(crate) struct InnerPipelineManager {
    pub(crate) shaders: Mutex<HashMap<String, ShaderCacheEntry>>,
    pub(crate) desc_layout: vk::DescriptorSetLayout,
    pub(crate) device: Arc<InnerDevice>,
}

impl InnerPipelineManager {
    pub(crate) fn new(device: Arc<InnerDevice>) -> InnerPipelineManager {
        let cache_dir = Path::new(".cache");

        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir).expect("Failed to create cache directory");
            println!(".cache directory created");
        }

        let shader_cache_path = cache_dir.join("shader_data.json");
        let files: HashMap<String, ShaderCacheEntry> = if shader_cache_path.exists() {
            let mut contents = String::new();
            File::open(&shader_cache_path).expect("Failed to open shader cache").read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            HashMap::new()
        };

        InnerPipelineManager {
            shaders: Mutex::new(files),
            desc_layout: device.bindless_descriptors.layout,
            device,
        }
    }

    pub(crate) fn get_spv_path(&self, slang_path: &str) -> Option<String> {
        let mut shaders = self.shaders.lock().unwrap();
        let path = Path::new(slang_path);

        // Get .slang file modification time
        let meta = fs::metadata(path).ok()?;
        let modified = meta.modified().ok()?;
        let timestamp = modified.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()).unwrap_or(0);

        // If in cache and timestamp matches → return cached path
        if let Some(entry) = shaders.get(slang_path) {
            if entry.timestamp == timestamp && Path::new(&entry.spv).exists() {
                return Some(entry.spv.clone());
            }
        }

        // Otherwise compile
        if let Err(e) = Self::compile_shader(path) {
            eprintln!("Failed to compile shader {}: {:?}", slang_path, e);
            return None;
        }

        // Construct spv path
        let spv_path = Path::new(".cache").join(path.file_name().unwrap()).with_extension("spv").to_string_lossy().to_string();

        // Update cache entry
        shaders.insert(
            slang_path.to_string(),
            ShaderCacheEntry {
                slang: slang_path.to_string(),
                spv: spv_path.clone(),
                timestamp,
            },
        );

        // Write updated cache
        let json_path = Path::new(".cache").join("shader_data.json");
        if let Ok(json) = serde_json::to_string_pretty(&*shaders) {
            if let Ok(mut file) = File::create(json_path) {
                let _ = file.write_all(json.as_bytes());
            }
        }

        Some(spv_path)
    }

    fn compile_shader(path: &Path) -> std::io::Result<()> {
        let output = Command::new("slangc")
            .arg(path)
            .arg("-o")
            .arg(Path::new(".cache").join(path.file_name().unwrap()).with_extension("spv")) // replaces .slang with .spv and also places the compiled shaders inside the .cache directory
            .output()?;

        if !output.status.success() {
            eprintln!("Failed to compile shader {:?}: {}", path, String::from_utf8_lossy(&output.stderr));
        } else {
            println!("Compiled shader {:?}", path);
        }

        Ok(())
    }
}

//// Pipeline creation ////
impl InnerPipelineManager {
    pub(crate) fn create_raster_pipeline_data(&self, raster_pipeline_desc: &RasterizationPipelineDescription) -> (vk::Pipeline, vk::PipelineLayout) {
        let vertex_shader_path = self
            .get_spv_path(raster_pipeline_desc.vertex_shader_path)
            .unwrap_or_else(|| panic!("Wrong vertex shader path provided"));

        let fragment_shader_path = self
            .get_spv_path(raster_pipeline_desc.fragment_shader_path)
            .unwrap_or_else(|| panic!("Wrong fragment shader path provided"));

        //Shaders
        let vert_code = InnerPipelineManager::read_spv_file(&vertex_shader_path);
        let frag_code = InnerPipelineManager::read_spv_file(&fragment_shader_path);

        let vert_module_create_info = vk::ShaderModuleCreateInfo::default().code(&vert_code);
        let frag_module_create_info = vk::ShaderModuleCreateInfo::default().code(&frag_code);

        let vert_module = unsafe { self.device.handle.create_shader_module(&vert_module_create_info, None).expect("Failed to create vertex shader module") };
        let frag_module = unsafe {
            self.device
                .handle
                .create_shader_module(&frag_module_create_info, None)
                .expect("Failed to create fragment shader module")
        };

        let entry_point = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_module).name(&entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(&entry_point),
        ];

        //Pipeline Layout
        let push_constant_ranges = [vk::PushConstantRange::default()
            .offset(raster_pipeline_desc.push_constants.offset)
            .size(raster_pipeline_desc.push_constants.size)
            .stage_flags(raster_pipeline_desc.push_constants.stage_flags.to_vk())];
        let layouts = [self.desc_layout];
        let layout_info = if raster_pipeline_desc.push_constants.size == 0 {
            vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts)
        } else {
            vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts).push_constant_ranges(&push_constant_ranges)
        };

        let pipeline_layout = unsafe { self.device.handle.create_pipeline_layout(&layout_info, None).expect("Failed to create pipeline layout") };

        //Vertex inpput
        let (vertex_input_binding, vertex_input_attributes) = raster_pipeline_desc.vertex_input.to_vk();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_input_binding)
            .vertex_attribute_descriptions(&vertex_input_attributes);

        //Brrr
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(raster_pipeline_desc.polygon_mode.to_vk_flag())
            .cull_mode(raster_pipeline_desc.cull_mode.to_vk_flag())
            .front_face(raster_pipeline_desc.front_face.to_vk_flag())
            .depth_bias_enable(false)
            .line_width(1.0);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(raster_pipeline_desc.depth_stencil.depth_test_enable)
            .depth_write_enable(raster_pipeline_desc.depth_stencil.depth_write_enable)
            .depth_compare_op(raster_pipeline_desc.depth_stencil.depth_compare_op.to_vk())
            .depth_bounds_test_enable(false)
            .stencil_test_enable(raster_pipeline_desc.depth_stencil.stencil_test_enable);

        let color_blend_attachment = if raster_pipeline_desc.alpha_blend_enable {
            vk::PipelineColorBlendAttachmentState {
                blend_enable: vk::TRUE,
                src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::RGBA,
            }
        } else {
            vk::PipelineColorBlendAttachmentState {
                blend_enable: vk::FALSE,
                src_color_blend_factor: vk::BlendFactor::ONE,
                dst_color_blend_factor: vk::BlendFactor::ZERO,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::RGBA,
            }
        };

        let arr = [color_blend_attachment];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default().logic_op_enable(false).attachments(&arr);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let color_formats = raster_pipeline_desc.outputs.color.iter().map(|f| f.to_vk_format()).collect::<Vec<vk::Format>>();

        //Dynamic rendering
        let mut dynamic_rendering_info = {
            let a = vk::PipelineRenderingCreateInfo::default().color_attachment_formats(color_formats.as_slice());
            let b = if raster_pipeline_desc.outputs.depth.is_some() {
                a.depth_attachment_format(raster_pipeline_desc.outputs.depth.clone().unwrap().to_vk_format())
            } else {
                a
            };

            let c = if raster_pipeline_desc.outputs.stencil.is_some() {
                b.stencil_attachment_format(raster_pipeline_desc.outputs.stencil.clone().unwrap().to_vk_format())
            } else {
                b
            };

            c
        };

        //Pipeline info
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .push_next(&mut dynamic_rendering_info);

        let pipeline = unsafe {
            self.device
                .handle
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        unsafe {
            self.device.handle.destroy_shader_module(vert_module, None);
            self.device.handle.destroy_shader_module(frag_module, None);
        }

        return (pipeline, pipeline_layout);
    }

    pub(crate) fn create_compute_pipeline(&self, compute_pipeline_desc: &ComputePipelineDescription) -> (vk::Pipeline, vk::PipelineLayout) {
        let shader = self.get_spv_path(compute_pipeline_desc.shader_path).unwrap_or_else(|| panic!("Wrong shader provided!!"));

        let shader_code = InnerPipelineManager::read_spv_file(&shader);

        let module_create_info = vk::ShaderModuleCreateInfo::default().code(shader_code.as_slice());

        let shader_module = unsafe { self.device.handle.create_shader_module(&module_create_info, None).expect("Failed to crate shader module") };

        // pipeline layout
        let push_constant_ranges = [vk::PushConstantRange::default()
            .offset(compute_pipeline_desc.push_constants.offset)
            .size(compute_pipeline_desc.push_constants.size)
            .stage_flags(compute_pipeline_desc.push_constants.stage_flags.to_vk())];
        let layouts = [self.desc_layout];
        let layout_info = if compute_pipeline_desc.push_constants.size == 0 {
            vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts)
        } else {
            vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts).push_constant_ranges(&push_constant_ranges)
        };

        let pipeline_layout = unsafe { self.device.handle.create_pipeline_layout(&layout_info, None).expect("Failed to create pipeline layout") };

        let entry_point = std::ffi::CString::new("main").unwrap();

        let shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(&entry_point);

        let pipeline_info = [vk::ComputePipelineCreateInfo::default().layout(pipeline_layout).stage(shader_stage_info)];

        let pipeline = unsafe {
            self.device
                .handle
                .create_compute_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
                .expect("Failed to create compute pipeline")
        }[0];

        return (pipeline, pipeline_layout);
    }
}

//// Helpers ////
impl InnerPipelineManager {
    fn read_spv_file(path: &str) -> Vec<u32> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path).expect("Failed to open shader file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read shader file");

        assert!(buffer.len() % 4 == 0, "SPIR-V file not aligned to 4 bytes");
        let words = unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() / 4) };
        words.to_vec()
    }
}

//==================== Rasterization Pipeline impl ==================== //

pub(crate) struct InnerRasterizationPipeline {
    pub(crate) handle: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) desc: RasterizationPipelineDescription,
    pub(crate) manager: Arc<InnerPipelineManager>,
}

impl Drop for InnerRasterizationPipeline {
    fn drop(&mut self) {
        unsafe {
            self.manager.device.handle.destroy_pipeline(self.handle, None);
            self.manager.device.handle.destroy_pipeline_layout(self.layout, None);
        }
    }
}

pub(crate) struct InnerComputePipeline {
    pub(crate) handle: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) manager: Arc<InnerPipelineManager>,
}

impl Drop for InnerComputePipeline {
    fn drop(&mut self) {
        unsafe {
            self.manager.device.handle.destroy_pipeline(self.handle, None);
            self.manager.device.handle.destroy_pipeline_layout(self.layout, None);
        }
    }
}
