use super::instance::PhysicalDevice;
use ash;

pub(crate) struct Device {
    pub(crate) handle: ash::Device,
    pub(crate) physical_device: PhysicalDevice,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        }
    }
}
