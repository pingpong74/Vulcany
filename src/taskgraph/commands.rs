use ash::vk;

use crate::{BufferID, ImageID, Pipeline, RasterizationPipeline, backend::device::InnerDevice};
use std::sync::Arc;

pub struct TaskGraphRecordingInterface {
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    pub(crate) queue_index: u32,
    pub(crate) queue: vk::Queue,
    pub(crate) device: Arc<InnerDevice>,
}

//// Public API with all recording functions
impl TaskGraphRecordingInterface {
    pub fn bind_rasterization_pipeline(&self, raster_pipeline: RasterizationPipeline) {
        unsafe {
            self.device.handle.cmd_bind_pipeline(
                self.command_buffers[0],
                vk::PipelineBindPoint::GRAPHICS,
                raster_pipeline.inner.handle,
            );
        };
    }

    pub fn bind_vertex_buffer(&self, buffer_id: BufferID, offset: u64) {
        let buffer_pool = self.device.buffer_pool.read().unwrap();
        let buffer_ref = buffer_pool.get_ref(buffer_id.id);

        let buffers = [buffer_ref.handle];
        let offsets = [offset];

        unsafe {
            self.device.handle.cmd_bind_vertex_buffers(
                self.command_buffers[0],
                0,
                &buffers,
                &offsets,
            );
        };
    }

    /// Need to add more index types
    pub fn bind_index_buffer(&self, buffer_id: BufferID, offset: u64) {
        let buffer_pool = self.device.buffer_pool.read().unwrap();
        let buffer_ref = buffer_pool.get_ref(buffer_id.id);

        unsafe {
            self.device.handle.cmd_bind_index_buffer(
                self.command_buffers[0],
                buffer_ref.handle,
                offset,
                vk::IndexType::UINT32,
            );
        };
    }

    pub fn begin_render_pass(&self) {
        let depth_attachment = vk::RenderingAttachmentInfo::default();

        let rendering_info = vk::RenderingInfo::default();

        unsafe {
            self.device
                .handle
                .cmd_begin_rendering(self.command_buffers[0], &rendering_info);
        };
    }

    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.handle.cmd_draw(
                self.command_buffers[0],
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        };
    }

    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.handle.cmd_draw_indexed(
                self.command_buffers[0],
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    pub fn end_render_pass(&self) {}
}

//// Private funcs for executing stuff
impl TaskGraphRecordingInterface {
    pub(crate) fn new(device: Arc<InnerDevice>) {}
}
