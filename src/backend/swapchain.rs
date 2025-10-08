use ash::vk;
use std::sync::Arc;

use crate::{Fence, ImageID, ImageViewID, Semaphore, Swapchain};

use crate::backend::device::InnerDevice;

pub(crate) struct InnerSwapchain {
    pub(crate) swapchain_loader: ash::khr::swapchain::Device,
    pub(crate) handle: vk::SwapchainKHR,
    pub(crate) curr_img_index: usize,
    pub(crate) images: Vec<ImageID>,
    pub(crate) image_views: Vec<ImageViewID>,
    pub(crate) device: Arc<InnerDevice>,
}

impl InnerSwapchain {
    pub(crate) fn acquire_image(
        &self,
        signal_semaphore: Option<&Semaphore>,
        signal_fence: Option<&Fence>,
    ) -> (ImageID, ImageViewID) {
        let acquire_info = vk::AcquireNextImageInfoKHR::default()
            .swapchain(self.handle)
            .timeout(u64::MAX)
            .semaphore(if signal_semaphore.is_some() {
                signal_semaphore.unwrap().handle()
            } else {
                vk::Semaphore::null()
            })
            .fence(if signal_fence.is_some() {
                signal_fence.unwrap().handle
            } else {
                vk::Fence::null()
            })
            .device_mask(1);

        let (index, _) = unsafe {
            self.swapchain_loader
                .acquire_next_image2(&acquire_info)
                .expect("Failed to acquire next image")
        };

        return (
            self.images[index as usize],
            self.image_views[index as usize],
        );
    }

    pub(crate) fn preset(&self, sempahore: &Semaphore) {
        let handle = [self.handle];
        let index = [self.curr_img_index as u32];
        let sem = [sempahore.handle()];

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&handle)
            .image_indices(&index)
            .wait_semaphores(&sem);

        unsafe {
            self.swapchain_loader
                .queue_present(self.device.graphics_queue, &present_info)
                .expect("Failed to preset image!!");
        }
    }

    pub(crate) fn resize() {}
}

impl Drop for InnerSwapchain {
    fn drop(&mut self) {
        for i in 0..self.image_views.len() {
            self.device
                .image_pool
                .write()
                .unwrap()
                .delete(self.images[i].id);

            self.device.destroy_image_view(self.image_views[i]);
        }

        unsafe {
            self.swapchain_loader.destroy_swapchain(self.handle, None);
        };
    }
}
