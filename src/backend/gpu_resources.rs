use std::thread::panicking;

use ash::vk;
use vk_mem::*;

pub(crate) struct BufferSlot {
    pub(crate) handle: vk::Buffer,
    pub(crate) allocation: Allocation,
    pub(crate) alloc_info: AllocationInfo,
}

pub(crate) struct ImageSlot {
    pub(crate) handle: vk::Image,
    pub(crate) allocation: Allocation,
    pub(crate) alloc_info: AllocationInfo,
    pub(crate) format: vk::Format,
}

pub(crate) struct ImageViewSlot {
    pub(crate) handle: vk::ImageView,
    pub(crate) parent_image: vk::Image,
}

pub(crate) struct SamplerSlot {
    pub(crate) handle: vk::Sampler,
}

//// Assinging 16 bits to each of the numbers, paging, index and version
//// <---- Filler bits -----> 16 paging 16 index 16 version
////
//// Actual creation and destruction happens on a device, this just manages the ids
////
//// TODO: Add multi threading

const MASK: u64 = 0xFFFF;

fn encode(page: u64, index: u64, version: u64) -> u64 {
    return (page << 32) | (index << 16) | version;
}

// return -> (Page, index, version)
fn decode(id: u64) -> (u64, u64, u64) {
    return ((id >> 32) & MASK, (id >> 16) & MASK, id & MASK);
}

fn decode_as_usize(id: u64) -> (usize, usize, u64) {
    return (
        ((id >> 32) & MASK) as usize,
        ((id >> 16) & MASK) as usize,
        (id & MASK),
    );
}

const PAGE_SIZE: usize = 10;

pub(crate) struct GpuResourcePool<Resource> {
    data: Vec<[(Option<Resource>, u64); PAGE_SIZE]>,
    free_indices: Vec<u64>,
    curr_page: usize,
    curr_index: usize,
}

impl<Resource> GpuResourcePool<Resource> {
    pub(crate) fn new() -> Self {
        return GpuResourcePool {
            data: vec![std::array::from_fn(|_| (None, 0))],
            free_indices: Vec::new(),
            curr_index: 0,
            curr_page: 0,
        };
    }

    pub(crate) fn add(&mut self, res: Resource) -> u64 {
        if self.free_indices.is_empty() {
            if self.curr_index == PAGE_SIZE {
                self.data.push(std::array::from_fn(|_| (None, 0)));
                self.curr_index = 0;
                self.curr_page += 1;
            }

            let id = encode(self.curr_page as u64, self.curr_index as u64, 0);

            self.data[self.curr_page][self.curr_index] = (Some(res), 0);

            self.curr_index += 1;

            return id;
        } else {
            let id = self.free_indices.pop().unwrap();

            let (page, index, version) = decode_as_usize(id);

            self.data[page][index] = (Some(res), version + 1);

            return encode(page as u64, index as u64, version + 1);
        }
    }

    pub(crate) fn delete(&mut self, id: u64) -> Resource {
        let (page, index, version) = decode_as_usize(id);

        let (res_opt, res_version) = &mut self.data[page][index];

        match res_opt.take() {
            Some(res) => {
                if *res_version == version {
                    self.data[page][index] = (None, version);
                    self.free_indices.push(id);

                    return res;
                } else {
                    panic!("Attempted to acess with invalid ID")
                }
            }
            None => {
                panic!("Attempted to acess with invalid ID")
            }
        }
    }

    pub(crate) fn get_ref(&self, id: u64) -> &Resource {
        let (page, index, version) = decode_as_usize(id);

        let (res_opt, res_version) = &self.data[page][index];

        match res_opt {
            Some(res) => {
                if *res_version == version {
                    return res;
                } else {
                    panic!("Attempted acess with invalid ID")
                }
            }
            None => {
                panic!("Attempted acess with invalid ID")
            }
        }
    }
}
