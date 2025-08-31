use std::collections::BTreeMap;

use super::allocation_info::{Allocation, MemoryBlock};
use super::gpu_allocator::GpuAllocator;

pub(crate) struct FreeListAllocator {
    device: ash::Device,
    memory_type: u32,
    memory_blocks: Vec<MemoryBlock>,
    block_size: ash::vk::DeviceSize,
}

//// Creation and Destruction ////
impl FreeListAllocator {
    pub(crate) fn new(
        device: ash::Device,
        memory_type: u32,
        block_size: ash::vk::DeviceSize,
    ) -> FreeListAllocator {
        return FreeListAllocator {
            device: device,
            memory_type: memory_type,
            memory_blocks: Vec::new(),
            block_size: block_size,
        };
    }

    pub(crate) fn destroy(&mut self) {
        for block in &self.memory_blocks {
            unsafe {
                self.device.free_memory(block.memory, None);
            };
        }
    }
}

//// Allocation calls ////
impl GpuAllocator for FreeListAllocator {
    fn allocate(
        &mut self,
        size: ash::vk::DeviceSize,
        alignment: ash::vk::DeviceSize,
        host_visible: bool,
    ) -> Allocation {
        if size >= self.block_size {
            panic!("[Free List Allocator] Too large of an allocation requesed")
        }

        for block in &mut self.memory_blocks {
            for (&offset, &range_size) in &block.free_ranges {
                let aligned_offset = Self::align_up(offset, alignment);
                let padding = aligned_offset - offset;

                if range_size >= size + padding {
                    block.free_ranges.remove(&offset);
                    if padding > 0 {
                        block.free_ranges.insert(offset, padding);
                    }
                    if range_size > size + padding {
                        block
                            .free_ranges
                            .insert(aligned_offset + size, range_size - size - padding);
                    }

                    let mapped_ptr = {
                        if host_visible {
                            let ptr = unsafe {
                                self.device
                                    .map_memory(
                                        block.memory,
                                        aligned_offset,
                                        size,
                                        ash::vk::MemoryMapFlags::empty(),
                                    )
                                    .expect("Failed to map memory")
                                    as *mut u8
                            };
                            Some(ptr)
                        } else {
                            None
                        }
                    };

                    return Allocation {
                        memory: block.memory,
                        offset: aligned_offset,
                        size,
                        mapped_ptr,
                    };
                }
            }
        }

        let mut new_block = self.create_new_block();
        new_block.free_ranges.remove(&0);
        new_block.free_ranges.insert(size, self.block_size - size);

        let mapped_ptr = {
            if host_visible {
                let ptr = unsafe {
                    self.device
                        .map_memory(new_block.memory, 0, size, ash::vk::MemoryMapFlags::empty())
                        .expect("Failed to map memory") as *mut u8
                };
                Some(ptr)
            } else {
                None
            }
        };

        let allocation = Allocation {
            memory: new_block.memory,
            offset: 0,
            size: size,
            mapped_ptr: mapped_ptr,
        };

        self.memory_blocks.push(new_block);

        return allocation;
    }

    fn free(&mut self, allocation: Allocation) {
        let block: &mut MemoryBlock = self
            .memory_blocks
            .iter_mut()
            .find(|b| b.memory == allocation.memory)
            .expect("[Free List allocator] Sent a buffer which was not allocated here!!");

        if allocation.mapped_ptr.is_some() {
            unsafe {
                self.device.unmap_memory(block.memory);
            };
        }

        block.free_ranges.insert(allocation.offset, allocation.size);

        //TODO Merge free ranges IDK HOW TO DO??
    }

    fn create_new_block(&mut self) -> MemoryBlock {
        let allocation_info = ash::vk::MemoryAllocateInfo::default()
            .allocation_size(self.block_size)
            .memory_type_index(self.memory_type);

        let memory = unsafe {
            self.device
                .allocate_memory(&allocation_info, None)
                .expect("Failed to allocate memory (From free list allocator)")
        };

        let mut free_ranges: BTreeMap<ash::vk::DeviceSize, ash::vk::DeviceSize> = BTreeMap::new();
        free_ranges.insert(0, self.block_size);

        return MemoryBlock {
            memory: memory,
            size: self.block_size,
            free_ranges: free_ranges,
        };
    }
}
