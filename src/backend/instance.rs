use super::device::Device;

use ash;

pub(crate) struct Instance {
    entry: ash::Entry,
    handle: ash::vk::Instance,
}

impl Instance {
    pub(crate) fn new() -> Instance {
        let entry = unsafe {
            ash::Entry::linked();
        };
    }
}
