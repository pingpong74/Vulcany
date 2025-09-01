use crate::backend::device::InnerDevice;
use crate::backend::instance::InnerInstance;
use std::sync::Arc;

use super::device::Device;

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

#[derive(Clone)]
pub struct Instance {
    pub(crate) inner: Arc<InnerInstance>,
}

impl Instance {
    pub fn new<W: HasDisplayHandle + HasWindowHandle>(
        instance_desc: &InstanceDescription<W>,
    ) -> Instance {
        let inner_instance = InnerInstance::new(instance_desc);
        return Instance {
            inner: Arc::new(inner_instance),
        };
    }

    pub fn create_device(&self, device_desc: &DeviceDescription) -> Device {
        let inner_device = self.inner.create_device(device_desc);

        return Device {
            inner: Arc::new(inner_device),
        };
    }
}
