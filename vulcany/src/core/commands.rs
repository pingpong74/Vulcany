use std::sync::Arc;

use ahash::HashMap;
use ash::vk;
use smallvec::SmallVec;

use crate::{Barrier, BufferCopyInfo, BufferID, CommandBufferUsage, ImageID, ImageViewID, IndexType, Pipeline, QueueType, RenderingBeginInfo, backend::device::InnerDevice};

/// Not thread safe!!
/// This is because normal vulkan command pools arent hread safe either
/// Hence it felt unnecessary to have an inner struct
pub struct CommandRecorder {
    pub(crate) handle: vk::CommandPool,
    pub(crate) commad_buffers: SmallVec<[vk::CommandBuffer; 2]>,
    pub(crate) exec_command_buffers: SmallVec<[vk::CommandBuffer; 2]>,
    pub(crate) current_commad_buffer: vk::CommandBuffer,
    pub(crate) queue_type: QueueType,
    pub(crate) remembered_image_ids: HashMap<ImageID, vk::Image>,
    pub(crate) remembered_buffer_ids: HashMap<BufferID, vk::Buffer>,
    pub(crate) remembered_image_view_ids: HashMap<ImageViewID, vk::ImageView>,
    pub(crate) device: Arc<InnerDevice>,
}

impl CommandRecorder {
    pub fn reset(&mut self) {
        unsafe {
            self.device
                .handle
                .reset_command_pool(self.handle, vk::CommandPoolResetFlags::empty())
                .expect("Failed to reset command pool");
        }

        self.commad_buffers.append(&mut self.exec_command_buffers);
    }

    pub fn begin_recording(&mut self, usage: CommandBufferUsage) {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(usage.to_vk_flags());

        if self.commad_buffers.is_empty() {
            self.current_commad_buffer = self.new_cmd_buffer();
        } else {
            self.current_commad_buffer = self.commad_buffers.pop().unwrap();
        }

        unsafe {
            self.device.handle.begin_command_buffer(self.current_commad_buffer, &begin_info).expect("Failed to begin cmd buffer!!!");
        }
    }

    pub fn end_recording(&mut self) -> ExecutableCommandBuffer {
        unsafe {
            self.device.handle.end_command_buffer(self.current_commad_buffer).expect("Failed to end cmd buffer!!!");
        }

        let return_buffer = self.current_commad_buffer;
        self.exec_command_buffers.push(return_buffer);
        self.current_commad_buffer = vk::CommandBuffer::null();

        return ExecutableCommandBuffer {
            handle: return_buffer,
            queue_type: self.queue_type,
        };
    }

    pub fn begin_rendering(&mut self, rendering_begin_info: &RenderingBeginInfo) {
        let mut color_attachment_info = SmallVec::<[vk::RenderingAttachmentInfo; 4]>::new();

        for color_attachement in &rendering_begin_info.color_attachments {
            let image_view = self.check_and_remeber_image_view_id(color_attachement.image_view);
            let resolve_image_view = if color_attachement.resolve_image_view.is_some() {
                self.check_and_remeber_image_view_id(color_attachement.resolve_image_view.unwrap())
            } else {
                vk::ImageView::null()
            };

            color_attachment_info.push(
                vk::RenderingAttachmentInfo::default()
                    .resolve_mode(color_attachement.resolve_mode.to_vk())
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
            .flags(rendering_begin_info.rendering_flags.to_vk())
            .color_attachments(color_attachment_info.as_slice())
            .layer_count(rendering_begin_info.layer_count)
            .view_mask(rendering_begin_info.view_mask)
            .render_area(vk::Rect2D {
                extent: vk::Extent2D {
                    width: rendering_begin_info.render_area.width,
                    height: rendering_begin_info.render_area.height,
                },
                offset: vk::Offset2D { x: 0, y: 0 },
            });

        let depth_attachment_info: vk::RenderingAttachmentInfo;
        let stencil_attachment_info: vk::RenderingAttachmentInfo;

        // Adding the optinal depth and stencil attachment
        if rendering_begin_info.depth_attachment.is_some() {
            let depth_attachment = rendering_begin_info.depth_attachment.as_ref().unwrap();

            let image_view = self.check_and_remeber_image_view_id(depth_attachment.image_view);
            let resolve_image_view = if depth_attachment.resolve_image_view.is_some() {
                self.check_and_remeber_image_view_id(depth_attachment.resolve_image_view.unwrap())
            } else {
                vk::ImageView::null()
            };

            depth_attachment_info = vk::RenderingAttachmentInfo::default()
                .resolve_mode(depth_attachment.resolve_mode.to_vk())
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
            let stencil_attachment = rendering_begin_info.stencil_attachment.as_ref().unwrap();

            let image_view = self.check_and_remeber_image_view_id(stencil_attachment.image_view);
            let resolve_image_view = if stencil_attachment.resolve_image_view.is_some() {
                self.check_and_remeber_image_view_id(stencil_attachment.resolve_image_view.unwrap())
            } else {
                vk::ImageView::null()
            };

            stencil_attachment_info = vk::RenderingAttachmentInfo::default()
                .resolve_mode(stencil_attachment.resolve_mode.to_vk())
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
            self.device.handle.cmd_begin_rendering(self.current_commad_buffer, &rendering_info);
        }
    }

    pub fn end_rendering(&self) {
        unsafe {
            self.device.handle.cmd_end_rendering(self.current_commad_buffer);
        }
    }

    //// Bind Commands ////
    pub fn set_viewport_and_scissor(&self, width: u32, height: u32) {
        unsafe {
            self.device.handle.cmd_set_viewport(
                self.current_commad_buffer,
                0,
                &[vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: width as f32,
                    height: height as f32,
                    max_depth: 1.0,
                    min_depth: 0.0,
                }],
            );

            self.device.handle.cmd_set_scissor(
                self.current_commad_buffer,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D { width: width, height: height },
                }],
            );
        }
    }

    pub fn set_push_constants<T: bytemuck::Pod>(&self, push_constants: &T, pipeline: Pipeline) {
        let data = bytemuck::bytes_of(push_constants);
        unsafe {
            self.device
                .handle
                .cmd_push_constants(self.current_commad_buffer, pipeline.get_layout(), vk::ShaderStageFlags::ALL, 0, data);
        }
    }

    pub fn bind_pipeline(&self, pipeline: &Pipeline) {
        unsafe {
            match pipeline {
                Pipeline::RasterizationPipeline(inner) => {
                    self.device.handle.cmd_bind_pipeline(self.current_commad_buffer, vk::PipelineBindPoint::GRAPHICS, inner.handle);
                    self.device.handle.cmd_bind_descriptor_sets(
                        self.current_commad_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        inner.layout,
                        0,
                        &[self.device.bindless_descriptors.set],
                        &[],
                    );
                }
                Pipeline::ComputePipeline(inner) => self.device.handle.cmd_bind_pipeline(self.current_commad_buffer, vk::PipelineBindPoint::COMPUTE, inner.handle),
            }
        }
    }

    pub fn bind_vertex_buffer(&mut self, buffer_id: BufferID, offset: u64) {
        let buffer = [self.check_and_remeber_buffer_id(buffer_id)];
        let offset = [offset];

        unsafe {
            self.device.handle.cmd_bind_vertex_buffers(self.current_commad_buffer, 0, &buffer, &offset);
        }
    }

    pub fn bind_index_buffer(&mut self, buffer_id: BufferID, offset: u64, index_type: IndexType) {
        let buffer = self.check_and_remeber_buffer_id(buffer_id);

        unsafe {
            self.device.handle.cmd_bind_index_buffer(self.current_commad_buffer, buffer, offset, index_type.to_vk_flag());
        }
    }

    //// Draw commands ////
    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.device.handle.cmd_draw(self.current_commad_buffer, vertex_count, instance_count, first_vertex, first_instance);
        };
    }

    pub fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        unsafe {
            self.device
                .handle
                .cmd_draw_indexed(self.current_commad_buffer, index_count, instance_count, first_index, vertex_offset, first_instance);
        }
    }

    //// Pipeline barriers and sync ////
    pub fn pipeline_barrier(&mut self, barriers: &[Barrier]) {
        let mut mem_barriers = SmallVec::<[vk::MemoryBarrier2; 4]>::new();
        let mut image_barriers = SmallVec::<[vk::ImageMemoryBarrier2; 4]>::new();
        let mut buffer_barriers = SmallVec::<[vk::BufferMemoryBarrier2; 4]>::new();

        for b in barriers {
            match b {
                Barrier::Memory(mem_barrier) => {
                    mem_barriers.push(
                        vk::MemoryBarrier2::default()
                            .src_stage_mask(mem_barrier.src_stage.to_vk())
                            .src_access_mask(mem_barrier.src_access.to_vk())
                            .dst_stage_mask(mem_barrier.dst_stage.to_vk())
                            .dst_access_mask(mem_barrier.dst_access.to_vk()),
                    );
                }
                Barrier::Image(img_barrier) => {
                    let img = self.check_and_remeber_image_id(img_barrier.image);

                    let subresource_range = vk::ImageSubresourceRange {
                        aspect_mask: img_barrier.aspect.to_vk_aspect(),
                        base_mip_level: img_barrier.base_mip,
                        level_count: img_barrier.level_count,
                        base_array_layer: img_barrier.base_layer,
                        layer_count: img_barrier.layer_count,
                    };

                    image_barriers.push(
                        vk::ImageMemoryBarrier2::default()
                            .src_stage_mask(img_barrier.src_stage.to_vk())
                            .src_access_mask(img_barrier.src_access.to_vk())
                            .dst_stage_mask(img_barrier.dst_stage.to_vk())
                            .dst_access_mask(img_barrier.dst_access.to_vk())
                            .old_layout(img_barrier.old_layout.to_vk_layout())
                            .new_layout(img_barrier.new_layout.to_vk_layout())
                            .image(img)
                            .subresource_range(subresource_range),
                    );
                }
                Barrier::Buffer(buffer_barrier) => {
                    let buf = self.check_and_remeber_buffer_id(buffer_barrier.buffer);
                    buffer_barriers.push(
                        vk::BufferMemoryBarrier2::default()
                            .src_stage_mask(buffer_barrier.src_stage.to_vk())
                            .src_access_mask(buffer_barrier.src_access.to_vk())
                            .dst_stage_mask(buffer_barrier.dst_stage.to_vk())
                            .dst_access_mask(buffer_barrier.dst_access.to_vk())
                            .buffer(buf)
                            .offset(buffer_barrier.offset)
                            .size(buffer_barrier.size),
                    );
                }
            }
        }

        let dep_info = vk::DependencyInfo::default()
            .memory_barriers(mem_barriers.as_slice())
            .image_memory_barriers(image_barriers.as_slice())
            .buffer_memory_barriers(buffer_barriers.as_slice());

        unsafe {
            self.device.handle.cmd_pipeline_barrier2(self.current_commad_buffer, &dep_info);
        }
    }

    //// Copy commands ////
    pub fn copy_buffer(&mut self, buffer_copy_info: &BufferCopyInfo) {
        let src_buffer = self.check_and_remeber_buffer_id(buffer_copy_info.src_buffer);
        let dst_buffer = self.check_and_remeber_buffer_id(buffer_copy_info.dst_buffer);

        let copy_region = vk::BufferCopy2::default().src_offset(0).dst_offset(0).size(buffer_copy_info.size);

        let copy_info = vk::CopyBufferInfo2::default().src_buffer(src_buffer).dst_buffer(dst_buffer).regions(std::slice::from_ref(&copy_region));

        unsafe {
            self.device.handle.cmd_copy_buffer2(self.current_commad_buffer, &copy_info);
        }
    }
}

impl CommandRecorder {
    fn check_and_remeber_image_id(&mut self, id: ImageID) -> vk::Image {
        match self.remembered_image_ids.get(&id) {
            Some(img) => img.clone(),
            None => {
                let img_pool = self.device.image_pool.read().unwrap();
                let img = img_pool.get_ref(id.id);
                self.remembered_image_ids.insert(id, img.handle);
                img.handle
            }
        }
    }

    fn check_and_remeber_buffer_id(&mut self, id: BufferID) -> vk::Buffer {
        match self.remembered_buffer_ids.get(&id) {
            Some(buff) => buff.clone(),
            None => {
                let buffer_pool = self.device.buffer_pool.read().unwrap();
                let buffer = buffer_pool.get_ref(id.id);
                self.remembered_buffer_ids.insert(id, buffer.handle);
                buffer.handle
            }
        }
    }

    fn check_and_remeber_image_view_id(&mut self, id: ImageViewID) -> vk::ImageView {
        match self.remembered_image_view_ids.get(&id) {
            Some(img_view) => img_view.clone(),
            None => {
                let pool = self.device.image_view_pool.read().unwrap();
                let img_view = pool.get_ref(id.id);
                self.remembered_image_view_ids.insert(id, img_view.handle);
                img_view.handle
            }
        }
    }

    pub(crate) fn new_cmd_buffer(&self) -> vk::CommandBuffer {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(self.handle)
            .level(vk::CommandBufferLevel::PRIMARY);

        let cmd_buffer = unsafe { self.device.handle.allocate_command_buffers(&alloc_info).expect("Failed to allocate command buffer") }[0];

        return cmd_buffer;
    }
}

impl Drop for CommandRecorder {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_command_pool(self.handle, None);
        }
    }
}

pub struct ExecutableCommandBuffer {
    pub(crate) handle: vk::CommandBuffer,
    pub(crate) queue_type: QueueType,
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
