use std::sync::Arc;

use ash::vk;

use crate::backend::device::InnerDevice;

pub(crate) struct InnerSwapchain {
    pub(crate) swapchain_loader: ash::khr::swapchain::Device,
    pub(crate) handle: vk::SwapchainKHR,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) image_views: Vec<vk::ImageView>,
    pub(crate) device: Arc<InnerDevice>,
}

impl Drop for InnerSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader.destroy_swapchain(self.handle, None);
        };
    }
}
