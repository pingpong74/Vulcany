use std::collections::BTreeMap;

pub(crate) struct Allocation {
    pub(crate) memory: ash::vk::DeviceMemory,
    pub(crate) offset: ash::vk::DeviceSize,
    pub(crate) size: ash::vk::DeviceSize,
    pub(crate) mapped_ptr: Option<*mut u8>,
}

pub(crate) struct MemoryBlock {
    pub(crate) memory: ash::vk::DeviceMemory,
    pub(crate) size: ash::vk::DeviceSize,
    pub(crate) free_ranges: BTreeMap<ash::vk::DeviceSize, ash::vk::DeviceSize>,
}
