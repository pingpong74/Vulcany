use crate::allocator::allocation_info::Allocation;

pub struct Buffer {
    handle: ash::vk::Buffer,
    allocation: Allocation,
}
