use super::device::InnerDevice;

use ash::vk;
use std::sync::Arc;

pub(crate) struct InnerBuffer {
    pub(crate) handle: vk::Buffer,
    pub(crate) allocation: vk_mem::Allocation,
    pub(crate) allocation_info: vk_mem::AllocationInfo,
    pub(crate) device: Arc<InnerDevice>,
}

impl Drop for InnerBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .allocator
                .destroy_buffer(self.handle, &mut self.allocation);
        };
    }
}
