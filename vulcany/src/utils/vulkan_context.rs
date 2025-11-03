use crate::*;
use delegate::delegate;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ops::Deref;

pub struct VulkanContext {
    instance: Instance,
    device: Device,
    swapchain: Swapchain,
}

impl VulkanContext {
    pub fn new<W: HasDisplayHandle + HasWindowHandle>(instance_desc: &InstanceDescription<W>, device_desc: &DeviceDescription, swapchain_desc: &SwapchainDescription) -> VulkanContext {
        let instance = Instance::new(instance_desc);
        let device = instance.create_device(device_desc);
        let swapchain = device.create_swapchain(swapchain_desc);

        return VulkanContext {
            instance: instance,
            device: device,
            swapchain: swapchain,
        };
    }
}

impl VulkanContext {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.device.wait_idle();
        let new_swapchain = self.device.recreate_swapchain(
            &SwapchainDescription {
                image_count: 3,
                width: width,
                height: height,
            },
            &self.swapchain,
        );
        let old_swapchain = std::mem::replace(&mut self.swapchain, new_swapchain);
        drop(old_swapchain);
    }
}

impl VulkanContext {
    delegate! {
        to self.device {
            pub fn create_buffer(&self, buffer_desc: &BufferDescription) -> BufferID;
            pub fn destroy_buffer(&self, id: BufferID);
        }
    }
}
