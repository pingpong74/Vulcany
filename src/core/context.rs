use crate::backend;

use ash;
use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[repr(u32)]
#[derive(Clone)]
pub enum ApiVersion {
    VK_API_1_0 = ash::vk::API_VERSION_1_0,
    VK_API_1_1 = ash::vk::API_VERSION_1_1,
    VK_API_1_2 = ash::vk::API_VERSION_1_2,
    VK_API_1_3 = ash::vk::API_VERSION_1_3,
}

pub struct InstanceDescription<W: HasDisplayHandle + HasWindowHandle> {
    pub api_version: ApiVersion,
    pub enable_validation_layers: bool,
    pub window: Arc<W>,
}

pub struct DeviceDescription {
    pub use_compute_queue: bool,
    pub use_transfer_queue: bool,
}

pub struct SwapchainDescription {
    pub image_count: u32,
    pub width: u32,
    pub height: u32,
}

pub struct BufferDescription {}

pub struct Context {
    swapchain: Arc<backend::swapchain::Swapchain>,
    device: Arc<backend::device::Device>,
    instance: Arc<backend::instance::Instance>,
}

impl Context {
    pub fn new<W: HasDisplayHandle + HasWindowHandle>(
        instance_desc: &InstanceDescription<W>,
        device_dec: &DeviceDescription,
        swapchain_desc: &SwapchainDescription,
    ) -> Context {
        let instance = backend::instance::Instance::new(instance_desc);
        let device = instance.create_device(device_dec);
        let swapchain =
            device.create_swapchain(swapchain_desc, &instance.handle, &instance.surface);

        return Context {
            instance: Arc::new(instance),
            device: Arc::new(device),
            swapchain: Arc::new(swapchain),
        };
    }

    pub fn create_buffer(desc: &BufferDescription) {}
}
