use super::device::InnerDevice;
use ash::vk::{self, IMG_FILTER_CUBIC_SPEC_VERSION};
use std::sync::Arc;

use crate::ImageViewDescription;

pub(crate) struct InnerImage {
    pub(crate) handle: vk::Image,
    pub(crate) allocation: vk_mem::Allocation,
    pub(crate) allocation_info: vk_mem::AllocationInfo,
    pub(crate) format: vk::Format,
    pub(crate) device: Arc<InnerDevice>,
}

impl InnerImage {
    pub(crate) fn create_image_view_data(
        &self,
        image_view_description: &ImageViewDescription,
    ) -> vk::ImageView {
        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .image(self.handle)
            .view_type(image_view_description.view_type.to_vk_type())
            .format(self.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(image_view_description.aspect.to_vk_aspect())
                    .base_mip_level(image_view_description.base_mip_level)
                    .level_count(image_view_description.layer_count)
                    .base_array_layer(image_view_description.base_array_layer)
                    .layer_count(image_view_description.layer_count),
            );

        return unsafe {
            self.device
                .handle
                .create_image_view(&image_view_create_info, None)
                .expect("Failed to create Image view")
        };
    }
}

impl Drop for InnerImage {
    fn drop(&mut self) {
        unsafe {
            self.device
                .allocator
                .destroy_image(self.handle, &mut self.allocation);
        };
    }
}

pub(crate) struct InnerImageView {
    pub(crate) handle: vk::ImageView,
    pub(crate) image: Arc<InnerImage>,
}

impl Drop for InnerImageView {
    fn drop(&mut self) {
        unsafe {
            self.image
                .device
                .handle
                .destroy_image_view(self.handle, None);
        }
    }
}

pub(crate) struct InnerSampler {
    pub(crate) handle: vk::Sampler,
    pub(crate) device: Arc<InnerDevice>,
}

impl Drop for InnerSampler {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_sampler(self.handle, None);
        }
    }
}
