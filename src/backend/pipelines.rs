use ash::vk;

use crate::backend::device::InnerDevice;
use std::sync::Arc;

use crate::RasterizationPipelineDescription;

pub(crate) struct InnerPipelineManager {
    pub(crate) desc_pool: vk::DescriptorPool,
    pub(crate) desc_layout: vk::DescriptorSetLayout,
    pub(crate) desc_set: vk::DescriptorSet,
    pub(crate) device: Arc<InnerDevice>,
}

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

        let mut dynamic_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(
                raster_pipeline_desc
                    .outputs
                    .color
                    .iter()
                    .map(|f| f.to_vk_format())
                    .collect::<Vec<vk::Format>>()
                    .as_slice(),
            );

        if raster_pipeline_desc.outputs.depth.is_some() {
            dynamic_rendering_info.depth_attachment_format(
                raster_pipeline_desc
                    .outputs
                    .depth
                    .as_ref()
                    .unwrap()
                    .to_vk_format(),
            );
        }
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
            .layout(pipeline_layout);

        let pipeline = unsafe {
            self.device
                .handle
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        return (pipeline, pipeline_layout);
    }
}

////Private funcs////
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
