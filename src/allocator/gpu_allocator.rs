use super::allocation_info::{Allocation, MemoryBlock};

pub(crate) trait GpuAllocator {
    fn allocate(
        &mut self,
        size: ash::vk::DeviceSize,
        alignment: ash::vk::DeviceSize,
        host_visible: bool,
    ) -> Allocation;

    fn free(&mut self, allocation: Allocation);

    fn create_new_block(&mut self) -> MemoryBlock;

    fn align_up(
        offset: ash::vk::DeviceSize,
        alignment: ash::vk::DeviceSize,
    ) -> ash::vk::DeviceSize {
        (offset + alignment - 1) & !(alignment - 1)
    }
}
