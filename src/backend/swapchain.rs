use ash::vk;
use std::sync::Arc;

use crate::{ImageID, ImageViewID, Swapchain};

use crate::backend::device::InnerDevice;

pub(crate) struct InnerSwapchain {
    pub(crate) swapchain_loader: ash::khr::swapchain::Device,
    pub(crate) handle: vk::SwapchainKHR,
    pub(crate) images: Vec<ImageID>,
    pub(crate) image_views: Vec<ImageViewID>,
    pub(crate) device: Arc<InnerDevice>,
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
