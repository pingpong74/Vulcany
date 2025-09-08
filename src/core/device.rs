use image::GenericImageView;

use crate::{
    Buffer, BufferDescription, Image, ImageDescription, PipelineManager, Sampler,
    SamplerDescription, Swapchain, SwapchainDescription, Texture, TextureDescription,
    backend::{
        buffer::InnerBuffer,
        device::InnerDevice,
        image::{InnerImage, InnerSampler},
        pipelines::InnerPipelineManager,
        swapchain::InnerSwapchain,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: Arc<InnerDevice>,
}

impl Device {
    pub fn create_swapchain(&self, swapchain_desc: &SwapchainDescription) -> Swapchain {
        let (loader, swapchain, images, image_views) =
            self.inner.create_swapchain_data(swapchain_desc);

        return Swapchain {
            inner: Arc::new(InnerSwapchain {
                handle: swapchain,
                swapchain_loader: loader,
                image_views: image_views,
                images: images,
                device: self.inner.clone(),
            }),
        };
    }

    pub fn create_buffer(&self, buffer_desc: &BufferDescription) -> Buffer {
        let (buffer, allocation, alloc_info) = self.inner.create_buffer_data(buffer_desc);

        return Buffer {
            inner: Arc::new(InnerBuffer {
                handle: buffer,
                allocation: allocation,
                allocation_info: alloc_info,
                device: self.inner.clone(),
            }),
        };
    }

    pub fn create_image(&self, image_desc: &ImageDescription) -> Image {
        let (image, allocation, allocation_info) = self.inner.create_image_data(image_desc);

        return Image {
            inner: Arc::new(InnerImage {
                handle: image,
                allocation: allocation,
                allocation_info: allocation_info,
                format: image_desc.format.to_vk_format(),
                device: self.inner.clone(),
            }),
        };
    }

    pub fn create_sampler(&self, sampler_desc: &SamplerDescription) -> Sampler {
        let sampler = self.inner.create_sampler(sampler_desc);

        return Sampler {
            inner: Arc::new(InnerSampler {
                handle: sampler,
                device: self.inner.clone(),
            }),
        };
    }

    pub fn create_texture(&self, texture_desc: TextureDescription) -> Texture {
        let img = image::open(texture_desc.path).expect("Failed to open image");
        let (width, height) = img.dimensions();
        let raw_data = img.into_bytes();

        unimplemented!()
    }

    pub fn create_pipeline_manager(&self) -> PipelineManager {
        let (pool, set, layout) = self.inner.create_pipeline_manager_data();

        return PipelineManager {
            inner: Arc::new(InnerPipelineManager {
                desc_pool: pool,
                desc_layout: layout,
                desc_set: set,
                device: self.inner.clone(),
            }),
        };
    }
}
