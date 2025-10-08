use std::sync::Arc;

use ash::vk;

use smallvec::SmallVec;

use crate::{
    Barrier, BufferCopyInfo, BufferID, CommandBufferLevel, CommandBufferUsage, IndexType,
    RasterizationPipeline, RenderingBeginInfo, backend::device::InnerDevice,
};
#[derive(Clone)]

pub struct CommandBuffer {
    pub(crate) handle: vk::CommandBuffer,
    pub(crate) device: Arc<InnerDevice>,
}

impl CommandBuffer {
    //// Begining and end functions
    pub fn begin_recording(&self, usage: CommandBufferUsage) {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(usage.to_vk_flags());

        unsafe {
            self.device
                .handle
                .begin_command_buffer(self.handle, &begin_info);
        }
    }

    pub fn end_recording(&self) {
        unsafe {
            self.device.handle.end_command_buffer(self.handle);
        }
    }

    pub fn begin_rendering(&self, rendering_begin_info: &RenderingBeginInfo) {
        let mut color_attachment_info = SmallVec::<[vk::RenderingAttachmentInfo; 4]>::new();

        let image_view_pool = self.device.image_view_pool.read().unwrap();

        for color_attachement in &rendering_begin_info.color_attachments {
            let image_view = image_view_pool
                .get_ref(color_attachement.image_view.id)
                .handle;
            let resolve_image_view = if color_attachement.resolve_image_view.is_some() {
                image_view_pool
                    .get_ref(color_attachement.resolve_image_view.unwrap().id)
                    .handle
            } else {
                vk::ImageView::null()
            };

            color_attachment_info.push(
                vk::RenderingAttachmentInfo::default()
                    .image_view(image_view)
                    .image_layout(color_attachement.image_layout.to_vk_layout())
                    .resolve_image_view(resolve_image_view)
                    .resolve_image_layout(color_attachement.resolve_image_layout.to_vk_layout())
                    .load_op(color_attachement.load_op.to_vk())
                    .store_op(color_attachement.store_op.to_vk())
                    .clear_value(color_attachement.clear_value.to_vk()),
            )
        }

        let mut rendering_info = vk::RenderingInfo::default()
            .color_attachments(color_attachment_info.as_slice())
            .layer_count(rendering_begin_info.layer_count)
            .view_mask(rendering_begin_info.view_mask);

        let mut depth_attachment_info: vk::RenderingAttachmentInfo;
        let mut stencil_attachment_info: vk::RenderingAttachmentInfo;

        // Adding the optinal depth and stencil attachment
        if rendering_begin_info.depth_attachment.is_some() {
            let depth_attachment = rendering_begin_info.depth_attachment.as_ref().unwrap();

            let image_view = image_view_pool
                .get_ref(depth_attachment.image_view.id)
                .handle;
            let resolve_image_view = if depth_attachment.resolve_image_view.is_some() {
                image_view_pool
                    .get_ref(depth_attachment.resolve_image_view.unwrap().id)
                    .handle
            } else {
                vk::ImageView::null()
            };

            depth_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_view(image_view)
                .image_layout(depth_attachment.image_layout.to_vk_layout())
                .resolve_image_view(resolve_image_view)
                .resolve_image_layout(depth_attachment.resolve_image_layout.to_vk_layout())
                .load_op(depth_attachment.load_op.to_vk())
                .store_op(depth_attachment.store_op.to_vk())
                .clear_value(depth_attachment.clear_value.to_vk());

            rendering_info = rendering_info.depth_attachment(&depth_attachment_info);
        }

        if rendering_begin_info.stencil_attachment.is_some() {
            let stencil_attachment = &rendering_begin_info.stencil_attachment.as_ref().unwrap();

            let image_view = image_view_pool
                .get_ref(stencil_attachment.image_view.id)
                .handle;
            let resolve_image_view = if stencil_attachment.resolve_image_view.is_some() {
                image_view_pool
                    .get_ref(stencil_attachment.resolve_image_view.unwrap().id)
                    .handle
            } else {
                vk::ImageView::null()
            };

            stencil_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_view(image_view)
                .image_layout(stencil_attachment.image_layout.to_vk_layout())
                .resolve_image_view(resolve_image_view)
                .resolve_image_layout(stencil_attachment.resolve_image_layout.to_vk_layout())
                .load_op(stencil_attachment.load_op.to_vk())
                .store_op(stencil_attachment.store_op.to_vk())
                .clear_value(stencil_attachment.clear_value.to_vk());

            rendering_info = rendering_info.stencil_attachment(&stencil_attachment_info);
        }

        unsafe {
            self.device
                .handle
                .cmd_begin_rendering(self.handle, &rendering_info);
        }
    }

    pub fn end_rendering(&self) {
        unsafe {
            self.device.handle.cmd_end_rendering(self.handle);
        }
    }

    //// Bind Commands ////
    pub fn bind_rasterization_pipeline(&self, pipeline: &RasterizationPipeline) {
        unsafe {
            self.device.handle.cmd_bind_pipeline(
                self.handle,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.inner.handle,
            );
        }
    }

    pub fn bind_vertex_buffer(&self, buffer_id: BufferID, offset: u64) {
        let buffer_pool = self.device.buffer_pool.read().unwrap();
        let buffer = [buffer_pool.get_ref(buffer_id.id).handle];
        let offset = [offset];

        unsafe {
            self.device
                .handle
                .cmd_bind_vertex_buffers(self.handle, 0, &buffer, &offset);
        }
    }

    pub fn bind_index_buffer(&self, buffer_id: BufferID, offset: u64, index_type: IndexType) {
        let buffer_pool = self.device.buffer_pool.read().unwrap();
        let buffer = buffer_pool.get_ref(buffer_id.id).handle;

        unsafe {
            self.device.handle.cmd_bind_index_buffer(
                self.handle,
                buffer,
                offset,
                index_type.to_vk_flag(),
            );
        }
    }

    //// Draw commands ////
    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.handle.cmd_draw(
                self.handle,
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
                self.handle,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    //// Pipeline barriers and sync ////
    pub fn pipeline_barrier(&self, barriers: &[Barrier]) {
        let mut mem_barriers = SmallVec::<[vk::MemoryBarrier2; 4]>::new();
        let mut image_barriers = SmallVec::<[vk::ImageMemoryBarrier2; 4]>::new();
        let mut buffer_barriers = SmallVec::<[vk::BufferMemoryBarrier2; 4]>::new();

        let image_pool = self.device.image_pool.read().unwrap();
        let buffer_pool = self.device.buffer_pool.read().unwrap();

        for b in barriers {
            match b {
                Barrier::Memory {
                    src_stage,
                    dst_stage,
                    src_access,
                    dst_access,
                } => {
                    mem_barriers.push(
                        vk::MemoryBarrier2::default()
                            .src_stage_mask(src_stage.to_vk())
                            .src_access_mask(src_access.to_vk())
                            .dst_stage_mask(dst_stage.to_vk())
                            .dst_access_mask(dst_access.to_vk()),
                    );
                }
                Barrier::Image {
                    image,
                    old_layout,
                    new_layout,
                    src_stage,
                    dst_stage,
                    src_access,
                    dst_access,
                    base_mip,
                    level_count,
                    base_layer,
                    layer_count,
                } => {
                    let img = image_pool.get_ref(image.id);

                    let aspect_mask = match img.format {
                        vk::Format::D32_SFLOAT => vk::ImageAspectFlags::DEPTH,
                        vk::Format::D32_SFLOAT_S8_UINT => {
                            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
                        }
                        vk::Format::S8_UINT => vk::ImageAspectFlags::STENCIL,
                        _ => vk::ImageAspectFlags::COLOR,
                    };

                    let subresource_range = vk::ImageSubresourceRange {
                        aspect_mask,
                        base_mip_level: *base_mip,
                        level_count: *level_count,
                        base_array_layer: *base_layer,
                        layer_count: *layer_count,
                    };

                    image_barriers.push(
                        vk::ImageMemoryBarrier2::default()
                            .src_stage_mask(src_stage.to_vk())
                            .src_access_mask(src_access.to_vk())
                            .dst_stage_mask(dst_stage.to_vk())
                            .dst_access_mask(dst_access.to_vk())
                            .old_layout(old_layout.to_vk_layout())
                            .new_layout(new_layout.to_vk_layout())
                            .image(img.handle)
                            .subresource_range(subresource_range),
                    );
                }
                Barrier::Buffer {
                    buffer,
                    src_stage,
                    dst_stage,
                    src_access,
                    dst_access,
                    offset,
                    size,
                } => {
                    let buf = buffer_pool.get_ref(buffer.id);
                    buffer_barriers.push(
                        vk::BufferMemoryBarrier2::default()
                            .src_stage_mask(src_stage.to_vk())
                            .src_access_mask(src_access.to_vk())
                            .dst_stage_mask(dst_stage.to_vk())
                            .dst_access_mask(dst_access.to_vk())
                            .buffer(buf.handle)
                            .offset(*offset)
                            .size(*size),
                    );
                }
            }
        }

        let dep_info = vk::DependencyInfo::default()
            .memory_barriers(mem_barriers.as_slice())
            .image_memory_barriers(image_barriers.as_slice())
            .buffer_memory_barriers(buffer_barriers.as_slice());

        unsafe {
            self.device
                .handle
                .cmd_pipeline_barrier2(self.handle, &dep_info);
        }
    }

    //// Copy commands ////
    pub fn copy_buffer(&self, buffer_copy_info: &BufferCopyInfo) {
        let buffer_pool = self.device.buffer_pool.read().unwrap();

        let src_buffer = buffer_pool.get_ref(buffer_copy_info.src_buffer.id).handle;
        let dst_buffer = buffer_pool.get_ref(buffer_copy_info.dst_buffer.id).handle;

        let copy_region = vk::BufferCopy2::default()
            .src_offset(0)
            .dst_offset(0)
            .size(buffer_copy_info.size);

        let copy_info = vk::CopyBufferInfo2::default()
            .src_buffer(src_buffer)
            .dst_buffer(dst_buffer)
            .regions(std::slice::from_ref(&copy_region));

        unsafe {
            self.device.handle.cmd_copy_buffer2(self.handle, &copy_info);
        }
    }
}

#[derive(Clone, Copy)]
pub struct Fence {
    pub(crate) handle: vk::Fence,
}

#[derive(Clone, Copy)]
pub struct BinarySemaphore {
    pub(crate) handle: vk::Semaphore,
}

#[derive(Clone, Copy)]
pub struct TimelineSemaphore {
    pub(crate) handle: vk::Semaphore,
}

#[derive(Clone, Copy)]
pub enum Semaphore {
    Binary(BinarySemaphore),
    Timeline(TimelineSemaphore),
}

impl Semaphore {
    pub(crate) const fn handle(&self) -> vk::Semaphore {
        return match self {
            Self::Binary(b) => b.handle,
            Self::Timeline(t) => t.handle,
        };
    }
}
