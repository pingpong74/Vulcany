use crate::{
    BufferDescription, BufferID, ImageDescription, SamplerDescription, SwapchainDescription,
    backend::{
        gpu_resources::{BufferSlot, GpuResourcePool},
        instance::InnerInstance,
        pipelines::InnerPipelineManager,
    },
};

use super::instance::PhysicalDevice;
use ash::{self, vk};
use std::sync::{Arc, RwLock};
use vk_mem::*;

pub(crate) struct InnerDevice {
    pub(crate) allocator: Allocator,
    pub(crate) handle: ash::Device,
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) instance: Arc<InnerInstance>,

    //Pools for various gpu resources
    pub(crate) buffer_pool: RwLock<GpuResourcePool<BufferSlot>>,
}

// Swapchain Creation //
impl InnerDevice {
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

    pub(crate) fn create_swapchain_data(
        &self,
        swapchain_description: &SwapchainDescription,
    ) -> (
        ash::khr::swapchain::Device,
        vk::SwapchainKHR,
        Vec<vk::Image>,
        Vec<vk::ImageView>,
    ) {
        let swapchain_loader =
            ash::khr::swapchain::Device::new(&self.instance.handle, &self.handle);

        let support = &self.physical_device.swapchain_support;

        let extent = InnerDevice::choose_extent(
            &support.capabilities,
            swapchain_description.width,
            swapchain_description.height,
        );
        let present_mode = InnerDevice::choose_present_mode(&support.present_modes);
        let surface_format = InnerDevice::choose_surface_format(&support.formats);

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
            .surface(self.instance.surface.handle)
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

        return (swapchain_loader, swapchain, images, image_views);
    }
}

// Buffer //
impl InnerDevice {
    pub(crate) fn create_buffer(&self, buffer_desc: &BufferDescription) -> BufferID {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .usage(buffer_desc.usage.to_vk_flag())
            .size(buffer_desc.size);

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: buffer_desc.memory_type.to_vk_flag(),
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            self.allocator
                .create_buffer(&buffer_create_info, &allocation_create_info)
                .expect("Failed to create buffer ")
        };

        let alloc_info = self.allocator.get_allocation_info(&allocation);

        let id = self.buffer_pool.write().unwrap().add(BufferSlot {
            handle: buffer,
            allocation: allocation,
            alloc_info: alloc_info,
        });

        return BufferID { id: id };
    }

    pub(crate) fn destroy_buffer(&self, id: u64) {
        let mut res = self.buffer_pool.write().unwrap().delete(id);

        unsafe {
            self.allocator
                .destroy_buffer(res.handle, &mut res.allocation);
        }
    }
}

// Image, Image View and Sampler //
/*impl InnerDevice {
    pub(crate) fn create_image_data(
        &self,
        image_desc: &ImageDescription,
    ) -> (vk::Image, Allocation) {
        let image_create_info = vk::ImageCreateInfo::default()
            .usage(image_desc.usage.to_vk_flag())
            .extent(vk::Extent3D {
                height: image_desc.height,
                width: image_desc.width,
                depth: image_desc.depth,
            })
            .format(image_desc.format.to_vk_format())
            .array_layers(image_desc.array_layers)
            .mip_levels(image_desc.mip_levels)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .image_type(vk::ImageType::TYPE_2D)
            .samples(image_desc.samples.to_vk_flags())
            .tiling(vk::ImageTiling::OPTIMAL);

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: image_desc.memory_type.to_vk_flag(),
            ..Default::default()
        };

        let (image, allocation) = unsafe {
            self.allocator
                .create_image(&image_create_info, &allocation_create_info)
                .expect("Failed to create image")
        };

        let alloc_info = self.allocator.get_allocation_info(&allocation);

        return (image, allocation, alloc_info);
    }

    pub(crate) fn create_sampler(&self, sampler_desc: &SamplerDescription) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(sampler_desc.mag_filter.to_vk())
            .min_filter(sampler_desc.min_filter.to_vk())
            .mipmap_mode(sampler_desc.mipmap_mode.to_vk())
            .address_mode_u(sampler_desc.address_mode_u.to_vk())
            .address_mode_v(sampler_desc.address_mode_v.to_vk())
            .address_mode_w(sampler_desc.address_mode_w.to_vk())
            .mip_lod_bias(sampler_desc.mip_lod_bias)
            .anisotropy_enable(sampler_desc.max_anisotropy.is_some())
            .max_anisotropy(sampler_desc.max_anisotropy.unwrap_or(1.0))
            .compare_enable(sampler_desc.compare_op.is_some())
            .compare_op(
                sampler_desc
                    .compare_op
                    .map(|c| c.to_vk())
                    .unwrap_or(vk::CompareOp::ALWAYS),
            )
            .min_lod(sampler_desc.min_lod)
            .max_lod(sampler_desc.max_lod)
            .border_color(sampler_desc.border_color.to_vk())
            .unnormalized_coordinates(sampler_desc.unnormalized_coordinates);

        let handle = unsafe {
            self.handle
                .create_sampler(&create_info, None)
                .expect("Failed to create sampler")
        };

        return handle;
    }
}*/

// Pipeline Manager //
impl InnerDevice {
    //TODO: Need to find max supported and then fill in the data
    pub(crate) fn create_pipeline_manager_data(
        &self,
    ) -> (
        vk::DescriptorPool,
        vk::DescriptorSet,
        vk::DescriptorSetLayout,
    ) {
        let max_textures = 100;
        let max_buffers = 100;

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_textures,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: max_buffers,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: max_buffers,
            },
        ];

        let pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(10)
            .pool_sizes(&pool_sizes);

        let descriptor_pool = unsafe {
            self.handle
                .create_descriptor_pool(&pool_create_info, None)
                .expect("Failed to create bindless descriptor pool")
        };

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(max_textures)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(max_buffers)
                .stage_flags(vk::ShaderStageFlags::ALL),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(max_buffers)
                .stage_flags(vk::ShaderStageFlags::ALL),
        ];

        let binding_flags = [
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
        ];

        let mut binding_flags_info =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .push_next(&mut binding_flags_info)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&bindings);

        let bindless_set_layout = unsafe {
            self.handle
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create bindless descriptor set layout")
        };

        let variable_counts = [10, 10, 10];
        let mut variable_count_info =
            vk::DescriptorSetVariableDescriptorCountAllocateInfo::default()
                .descriptor_counts(&variable_counts);

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&bindless_set_layout))
            .push_next(&mut variable_count_info);

        let bindless_set = unsafe {
            self.handle
                .allocate_descriptor_sets(&alloc_info)
                .expect("Failed to create bindless descriptor")
        }[0];

        InnerPipelineManager::compile_shaders_in_dir("examples/shaders");

        return (descriptor_pool, bindless_set, bindless_set_layout);
    }
}

impl Drop for InnerDevice {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(&mut self.allocator);
            self.handle.destroy_device(None);
        }
    }
}
