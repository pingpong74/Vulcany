use std::sync::Arc;

use ash::vk;

use crate::{CommandBuffer, CommandBufferLevel, backend::device::InnerDevice};

pub(crate) struct InnerCommandPool {
    handle: vk::CommandPool,
    device: Arc<InnerDevice>,
}

impl InnerCommandPool {
    pub(crate) fn allocate_command_buffer(&self, level: CommandBufferLevel) -> CommandBuffer {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(self.handle)
            .level(level.to_vk_flags());

        let cmd_buffer = unsafe {
            self.device
                .handle
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers")
        }[0];

        return CommandBuffer {
            handle: cmd_buffer,
            device: self.device.clone(),
        };
    }
}

impl Drop for InnerCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_command_pool(self.handle, None);
        };
    }
}
