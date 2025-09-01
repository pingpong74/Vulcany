use std::sync::Arc;

use crate::{
    BufferDescription, SwapchainDescription,
    allocator::free_list_allocator::FreeListAllocator,
    backend::{instance::Surface, swapchain::Swapchain},
};

use super::instance::PhysicalDevice;
use ash::{self, vk};

pub(crate) struct Device {
    pub(crate) handle: ash::Device,
    pub(crate) physical_device: PhysicalDevice,
}

// Swapchain Creation //
impl Device {
    fn choose_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        available_formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| available_formats[0])
    }

    fn choose_present_mode(available_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        if available_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }

    fn choose_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: u32,
        height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub(crate) fn create_swapchain(
        &self,
        swapchain_description: &SwapchainDescription,
        instance: &ash::Instance,
        surface: &Surface,
    ) -> Swapchain {
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, &self.handle);

        let support = &self.physical_device.swapchain_support;

        let extent = Device::choose_extent(
            &support.capabilities,
            swapchain_description.width,
            swapchain_description.height,
        );
        let present_mode = Device::choose_present_mode(&support.present_modes);
        let surface_format = Device::choose_surface_format(&support.formats);

        let graphics_family = self
            .physical_device
            .queue_families
            .graphics_family
            .expect("This shouldnt be possible lol");
        let present_family = self
            .physical_device
            .queue_families
            .presetation_family
            .expect("This shouldnt be possible lol");

        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.handle)
            .min_image_count(swapchain_description.image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let queue_family_indices = [graphics_family, present_family];

        if graphics_family != present_family {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        create_info = create_info
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain")
        };

        let images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get swapchain images")
        };

        let image_views: Vec<vk::ImageView> = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );

                unsafe {
                    self.handle
                        .create_image_view(&create_info, None)
                        .expect("Failed to create swapchain image view")
                }
            })
            .collect();

        return Swapchain {
            swapchain_loader: swapchain_loader,
            handle: swapchain,
            images: images,
            image_views: image_views,
            device: self.handle.clone(),
        };
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        }
    }
}
