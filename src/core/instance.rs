use crate::backend::{
    device::InnerDevice,
    gpu_resources::GpuResourcePool,
    instance::{InnerInstance, PhysicalDevice},
};
use std::sync::{Arc, RwLock};

use super::{
    definations::{DeviceDescription, InstanceDescription},
    device::Device,
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

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
        let (device, physical_device, allocator) = self.inner.create_device_data(device_desc);

        return Device {
            inner: Arc::new(InnerDevice {
                handle: device,
                physical_device: physical_device,
                allocator: allocator,
                instance: self.inner.clone(),

                //Resource Pools
                buffer_pool: RwLock::new(GpuResourcePool::new()),
                image_pool: RwLock::new(GpuResourcePool::new()),
                image_view_pool: RwLock::new(GpuResourcePool::new()),
                sampler_pool: RwLock::new(GpuResourcePool::new()),
            }),
        };
    }
}
