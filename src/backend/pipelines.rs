use ash::vk;

use crate::backend::device::InnerDevice;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use crate::RasterizationPipelineDescription;

// TODO
// Create a hash map which stores all .slag files as key and compiled .spv files as data.
// Add pipeline cache and also cache common VkPiplineLayouts
// Add a way to actually write stuff to descriptors (Last priority)
//
// TODO (small)
// Make sure where the cache is bwing created. right now for this 1 example its simple, no need.

pub(crate) struct InnerPipelineManager {
    pub(crate) desc_pool: vk::DescriptorPool,
    pub(crate) desc_layout: vk::DescriptorSetLayout,
    pub(crate) desc_set: vk::DescriptorSet,
    pub(crate) device: Arc<InnerDevice>,
}

//// Shader cache impl ////
impl InnerPipelineManager {
    pub(crate) fn compile_shaders_in_dir(shader_path: &str) {
        // Create cache directory if it doesnt exist
        let cache_dir = Path::new(".cache");

        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir).expect("Failed to create cache directory");
            println!(".cache directory created");
        } else {
            println!(".cache directory already exists");
        }

        // Create a shader cache file if not present, if it is present load it
        let shader_cache_path = Path::new(".cache/shader_data.json");

        let mut files: HashMap<String, u64> = if shader_cache_path.exists() {
            let mut contents = String::new();
            File::open(shader_cache_path)
                .expect("Failed to open shader cache")
                .read_to_string(&mut contents)
                .unwrap();
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Loop over all shaders in the directory
        for entry in
            fs::read_dir(Path::new(shader_path)).expect("Shader directory provided doesnt exist")
        {
            let entry = entry.expect("Err");
            let path = entry.path();

            if path.is_file() && path.extension().is_some() && path.extension().unwrap() == "slang"
            {
                let shader_str = path.to_string_lossy().to_string();

                // Get last modified timestamp of the file
                let modified = path
                    .metadata()
                    .unwrap()
                    .modified()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let needs_recompile = match files.get(&shader_str) {
                    Some(prev) if *prev >= modified => {
                        println!("Shader up to date: {}", shader_str);
                        false
                    }
                    _ => true,
                };

                if needs_recompile {
                    InnerPipelineManager::compile_shader(&path).expect("Failed to compile shader");
                    files.insert(shader_str, modified);
                }
            }
        }

        let json =
            serde_json::to_string_pretty(&files).expect("Failed to turn hash map into a string");
        std::fs::write(".cache/shader_data.json", json).expect("Failed to write to shader cache");
    }

    fn compile_shader(path: &Path) -> std::io::Result<()> {
        let output = Command::new("slangc")
            .arg(path)
            .arg("-o")
            .arg(
                Path::new(".cache")
                    .join(path.file_name().unwrap())
                    .with_extension("spv"),
            ) // replaces .slang with .spv and also places the compiled shaders inside the .cache directory
            .output()?;

        if !output.status.success() {
            eprintln!(
                "Failed to compile shader {:?}: {}",
                path,
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            println!("Compiled shader {:?}", path);
        }

        Ok(())
    }
}

//// Pipeline creation ////
impl InnerPipelineManager {
    pub(crate) fn create_raster_pipeline_data(
        &self,
        raster_pipeline_desc: &RasterizationPipelineDescription,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        //Shaders
        let vert_code =
            InnerPipelineManager::read_spv_file(&raster_pipeline_desc.vertex_shader_path);
        let frag_code =
            InnerPipelineManager::read_spv_file(&raster_pipeline_desc.fragment_shader_path);

        let vert_module_create_info = vk::ShaderModuleCreateInfo::default().code(&vert_code);
        let frag_module_create_info = vk::ShaderModuleCreateInfo::default().code(&frag_code);

        let vert_module = unsafe {
            self.device
                .handle
                .create_shader_module(&vert_module_create_info, None)
                .expect("Failed to create vertex shader module")
        };
        let frag_module = unsafe {
            self.device
                .handle
                .create_shader_module(&frag_module_create_info, None)
                .expect("Failed to create fragment shader module")
        };

        let entry_point = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(&entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(&entry_point),
        ];

        //Pipeline Layout
        let layouts = [self.desc_layout];
        let layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);

        let pipeline_layout = unsafe {
            self.device
                .handle
                .create_pipeline_layout(&layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        //Vertex inpput
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&raster_pipeline_desc.vertex_input.bindings)
            .vertex_attribute_descriptions(&raster_pipeline_desc.vertex_input.attributes);

        //Brrr
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(raster_pipeline_desc.polygon_mode.to_vk_flag())
            .cull_mode(raster_pipeline_desc.cull_mode.to_vk_flag())
            .front_face(raster_pipeline_desc.front_face.to_vk_flag())
            .depth_bias_enable(false)
            .line_width(raster_pipeline_desc.line_width);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(raster_pipeline_desc.depth_stencil.depth_test_enable)
            .depth_write_enable(raster_pipeline_desc.depth_stencil.depth_write_enable)
            .depth_compare_op(raster_pipeline_desc.depth_stencil.depth_compare_op)
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

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&arr);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let color_formats = raster_pipeline_desc
            .outputs
            .color
            .iter()
            .map(|f| f.to_vk_format())
            .collect::<Vec<vk::Format>>();

        //Dynamic rendering
        let mut dynamic_rendering_info = {
            let a = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(color_formats.as_slice());
            let b = if raster_pipeline_desc.outputs.depth.is_some() {
                a.depth_attachment_format(
                    raster_pipeline_desc
                        .outputs
                        .depth
                        .clone()
                        .unwrap()
                        .to_vk_format(),
                )
            } else {
                a
            };

            let c = if raster_pipeline_desc.outputs.stencil.is_some() {
                b.stencil_attachment_format(
                    raster_pipeline_desc
                        .outputs
                        .stencil
                        .clone()
                        .unwrap()
                        .to_vk_format(),
                )
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
        file.read_to_end(&mut buffer)
            .expect("Failed to read shader file");

        assert!(buffer.len() % 4 == 0, "SPIR-V file not aligned to 4 bytes");
        let words =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() / 4) };
        words.to_vec()
    }
}

impl Drop for InnerPipelineManager {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_descriptor_set_layout(self.desc_layout, None);
            self.device
                .handle
                .free_descriptor_sets(self.desc_pool, &[self.desc_set])
                .expect("Failed to destroy desc set");
            self.device
                .handle
                .destroy_descriptor_pool(self.desc_pool, None);
        };
    }
}

//==================== Rasterization Pipeline impl ==================== //

pub(crate) struct InnerRasterizationPipeline {
    pub(crate) handle: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) manager: Arc<InnerPipelineManager>,
}

impl Drop for InnerRasterizationPipeline {
    fn drop(&mut self) {
        unsafe {
            self.manager
                .device
                .handle
                .destroy_pipeline(self.handle, None);
        }
    }
}
