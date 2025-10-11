use image::GenericImageView;

use crate::{
    BinarySemaphore, BufferDescription, BufferID, CommandBuffer, CommandBufferLevel, Fence,
    ImageDescription, ImageID, ImageViewDescription, ImageViewID, PipelineManager, QueueSubmitInfo,
    QueueType, SamplerDescription, SamplerID, Semaphore, Swapchain, SwapchainDescription,
    TimelineSemaphore,
    backend::{device::InnerDevice, pipelines::InnerPipelineManager, swapchain::InnerSwapchain},
};
use std::sync::{Arc, Mutex, atomic::AtomicUsize};

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: Arc<InnerDevice>,
}

//Swapchain Impl//
impl Device {
    pub fn create_swapchain(&self, swapchain_desc: &SwapchainDescription) -> Swapchain {
        let (loader, swapchain, images, image_views) = self
            .inner
            .create_swapchain_data(swapchain_desc, ash::vk::SwapchainKHR::null());

        return Swapchain {
            inner: Arc::new(InnerSwapchain {
                handle: swapchain,
                swapchain_loader: loader,
                curr_img_index: AtomicUsize::new(0),
                image_views: image_views,
                images: images,
                device: self.inner.clone(),
            }),
        };
    }

    pub fn recreate_swapchain(
        &self,
        swapchain_desc: &SwapchainDescription,
        old_swapchain: &Swapchain,
    ) -> Swapchain {
        let (loader, swapchain, images, image_views) = self
            .inner
            .create_swapchain_data(swapchain_desc, old_swapchain.inner.handle);

        return Swapchain {
            inner: Arc::new(InnerSwapchain {
                handle: swapchain,
                swapchain_loader: loader,
                curr_img_index: AtomicUsize::new(0),
                image_views: image_views,
                images: images,
                device: self.inner.clone(),
            }),
        };
    }
}

// Buffer //
impl Device {
    pub fn create_buffer(&self, buffer_desc: &BufferDescription) -> BufferID {
        return self.inner.create_buffer(buffer_desc);
    }

    pub fn destroy_buffer(&self, id: BufferID) {
        self.inner.destroy_buffer(id);
    }

    pub fn write_data_to_buffer<T: Copy>(&self, buffer_id: BufferID, data: &[T]) {
        self.inner.write_data_to_buffer(buffer_id, data);
    }
}

// Image //
impl Device {
    pub fn create_image(&self, image_desc: &ImageDescription) -> ImageID {
        return self.inner.create_image(image_desc);
    }

    pub fn destroy_image(&self, image_id: ImageID) {
        self.inner.destroy_image(image_id);
    }
}

// Image View //
impl Device {
    pub fn create_image_view(
        &self,
        image_id: ImageID,
        image_view_desc: &ImageViewDescription,
    ) -> ImageViewID {
        return self.inner.create_image_view(image_id, image_view_desc);
    }

    pub fn destroy_image_view(&self, image_view_id: ImageViewID) {
        self.inner.destroy_image_view(image_view_id);
    }
}

// Pipeline Manager //
impl Device {
    pub fn create_pipeline_manager(&self, shader_directory: &str) -> PipelineManager {
        let (pool, set, layout) = self.inner.create_pipeline_manager_data(shader_directory);

        return PipelineManager {
            inner: Arc::new(InnerPipelineManager {
                shader_directory: shader_directory.to_string(),
                desc_pool: pool,
                desc_layout: layout,
                desc_set: set,
                device: self.inner.clone(),
            }),
        };
    }
}

// Command buffer //
impl Device {
    pub fn allocate_command_buffer(
        &self,
        level: CommandBufferLevel,
        queue_type: QueueType,
    ) -> CommandBuffer {
        return CommandBuffer {
            handle: self.inner.allocate_command_buffers(level, queue_type),
            queue_type,
            device: self.inner.clone(),
        };
    }

    pub fn free_command_buffer(&self, cmd_buffer: CommandBuffer) {
        self.inner.free_command_buffer(cmd_buffer);
    }

    pub fn reset_command_pool(&self, queue_type: QueueType) {
        self.inner.reset_command_pool(queue_type);
    }
}

// Sync //
impl Device {
    pub fn create_fence(&self, signaled: bool) -> Fence {
        return Fence {
            handle: self.inner.create_fence(signaled),
        };
    }

    pub fn create_binary_semaphore(&self) -> Semaphore {
        return Semaphore::Binary(BinarySemaphore {
            handle: self.inner.create_binary_semaphore(),
        });
    }

    pub fn create_timeline_semaphore(&self) -> Semaphore {
        return Semaphore::Timeline(TimelineSemaphore {
            handle: self.inner.create_timeline_semaphore(),
        });
    }

    pub fn wait_fence(&self, fence: Fence) {
        self.inner.wait_fence(fence);
    }

    pub fn reset_fence(&self, fence: Fence) {
        self.inner.reset_fence(fence);
    }

    pub fn destroy_fence(&self, fence: Fence) {
        self.inner.destroy_fence(fence);
    }

    pub fn destroy_semaphore(&self, semaphore: Semaphore) {
        self.inner.destroy_semaphore(semaphore);
    }
}

// Queue submissions
impl Device {
    pub fn submit(&self, submit_info: &QueueSubmitInfo) {
        self.inner.submit(submit_info);
    }

    pub fn wait_idle(&self) {
        self.inner.wait_idle();
    }

    pub fn wait_queue(&self, queue_type: QueueType) {
        self.inner.wait_queue(queue_type);
    }
}
