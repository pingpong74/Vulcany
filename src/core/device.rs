use ahash::{HashMap, HashMapExt};
use ash::vk;
use crossbeam::queue::ArrayQueue;
use smallvec::smallvec;

use crate::{
    BinarySemaphore, BufferDescription, BufferID, BufferWriteInfo, CommandRecorder, Fence, ImageDescription, ImageID, ImageViewDescription, ImageViewID, ImageWriteInfo, PipelineManager,
    QueueSubmitInfo, QueueType, SamplerWriteInfo, Semaphore, Swapchain, SwapchainDescription, TimelineSemaphore,
    backend::{device::InnerDevice, pipelines::InnerPipelineManager, swapchain::InnerSwapchain},
};
use std::sync::{Arc, atomic::AtomicUsize};

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: Arc<InnerDevice>,
}

//Swapchain Impl//
impl Device {
    pub fn create_swapchain(&self, swapchain_desc: &SwapchainDescription) -> Swapchain {
        let (loader, swapchain, images, image_views) = self.inner.create_swapchain_data(swapchain_desc, ash::vk::SwapchainKHR::null());

        let (image_semapgores, present_semaphore) = {
            let mut t: Vec<Semaphore> = vec![];
            let mut n: Vec<Semaphore> = vec![];

            for _ in 0..swapchain_desc.image_count {
                t.push(self.create_binary_semaphore());
                n.push(self.create_binary_semaphore());
            }

            (t, n)
        };

        return Swapchain {
            inner: Arc::new(InnerSwapchain {
                handle: swapchain,
                swapchain_loader: loader,
                curr_img_indeices: ArrayQueue::new(swapchain_desc.image_count as usize),
                image_views: image_views,
                images: images,
                image_semaphore: image_semapgores,
                preset_semaphore: present_semaphore,
                timeline: AtomicUsize::new(0),
                device: self.inner.clone(),
            }),
        };
    }

    pub fn recreate_swapchain(&self, swapchain_desc: &SwapchainDescription, old_swapchain: &Swapchain) -> Swapchain {
        let (loader, swapchain, images, image_views) = self.inner.create_swapchain_data(swapchain_desc, old_swapchain.inner.handle);

        let (image_semapgores, present_semaphore) = {
            let mut t: Vec<Semaphore> = vec![];
            let mut n: Vec<Semaphore> = vec![];

            for _ in 0..swapchain_desc.image_count {
                t.push(self.create_binary_semaphore());
                n.push(self.create_binary_semaphore());
            }

            (t, n)
        };

        return Swapchain {
            inner: Arc::new(InnerSwapchain {
                handle: swapchain,
                swapchain_loader: loader,
                curr_img_indeices: ArrayQueue::new(swapchain_desc.image_count as usize),
                image_views: image_views,
                images: images,
                image_semaphore: image_semapgores,
                preset_semaphore: present_semaphore,
                timeline: AtomicUsize::new(0),
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
    pub fn create_image_view(&self, image_id: ImageID, image_view_desc: &ImageViewDescription) -> ImageViewID {
        return self.inner.create_image_view(image_id, image_view_desc);
    }

    pub fn destroy_image_view(&self, image_view_id: ImageViewID) {
        self.inner.destroy_image_view(image_view_id);
    }
}

impl Device {
    pub fn write_buffer(&self, buffer_write_info: &BufferWriteInfo) {
        self.inner.write_buffer(buffer_write_info);
    }

    pub fn write_image(&self, image_write_info: &ImageWriteInfo) {
        self.inner.write_image(image_write_info);
    }

    pub fn write_sampler(&self, sampler_write_info: &SamplerWriteInfo) {
        self.inner.write_sampler(sampler_write_info);
    }
}

// Pipeline Manager //
impl Device {
    pub fn create_pipeline_manager(&self, shader_directory: &str) -> PipelineManager {
        InnerPipelineManager::compile_shaders_in_dir(shader_directory);

        return PipelineManager {
            inner: Arc::new(InnerPipelineManager {
                shader_directory: shader_directory.to_string(),
                desc_layout: self.inner.bindless_descriptors.layout,
                device: self.inner.clone(),
            }),
        };
    }
}

// Command buffer //
impl Device {
    pub fn create_command_recorder(&self, queue_type: QueueType) -> CommandRecorder {
        return CommandRecorder {
            handle: self.inner.createcmd_recorder_data(queue_type),
            commad_buffers: smallvec![],
            exec_command_buffers: smallvec![],
            current_commad_buffer: vk::CommandBuffer::null(),
            queue_type: queue_type,
            remembered_image_ids: HashMap::new(),
            remembered_buffer_ids: HashMap::new(),
            remembered_image_view_ids: HashMap::new(),
            device: self.inner.clone(),
        };
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
