use crate::{
    backend::device::InnerDevice,
    core::{instance::Instance, swapchain::Swapchain},
};
use std::sync::Arc;

pub struct BufferDescription {}

pub struct SwapchainDescription {
    pub image_count: u32,
    pub width: u32,
    pub height: u32,
    pub instance: Instance,
}

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: Arc<InnerDevice>,
}

impl Device {
    pub fn create_swapchain(&self, swapchain_desc: &SwapchainDescription) -> Swapchain {
        let inner_swapchain = self.inner.create_swapchain(
            swapchain_desc,
            &swapchain_desc.instance.inner.handle,
            &swapchain_desc.instance.inner.surface,
        );

        return Swapchain {
            inner: Arc::new(inner_swapchain),
        };
    }
}
