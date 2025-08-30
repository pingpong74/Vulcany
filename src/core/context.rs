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

pub struct DeviceDescription {}

pub struct SwapchainDescription {}

pub struct Context {
    device: Arc<backend::device::Device>,
    instance: Arc<backend::instance::Instance>,
    //swapchain: Option<Arc<backend::swapchain::Swapchain>>,
}

impl Context {
    pub fn new<W: HasDisplayHandle + HasWindowHandle>(
        instance_desc: &InstanceDescription<W>,
        device_dec: &DeviceDescription,
        swapchain_desc: Option<SwapchainDescription>,
    ) -> Context {
        let instance = backend::instance::Instance::new(instance_desc);
        let device = instance.create_device(device_dec);

        return Context {
            instance: Arc::new(instance),
            device: Arc::new(device),
            //swapchain: None,
        };
    }
}
