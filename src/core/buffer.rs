use std::sync::Arc;

use crate::{allocator::allocation_info::Allocation, backend::device::InnerDevice};

pub struct Buffer {
    handle: ash::vk::Buffer,
    allocation: Allocation,
    device: Arc<InnerDevice>,
}
