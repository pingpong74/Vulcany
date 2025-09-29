use image::GenericImageView;

use crate::{
    BufferDescription, BufferID, ImageDescription, ImageID, PipelineManager, SamplerDescription,
    SamplerID, Swapchain, SwapchainDescription,
    backend::{device::InnerDevice, pipelines::InnerPipelineManager, swapchain::InnerSwapchain},
};
use std::sync::Arc;

//We need to swwitch to an ID system for buffers, images and other gpu resources now.
//
// Have a Gpu resource pool class which will handle all this stuff
//
// Have a create and destroy function for each resource, need to sacrifice RAII

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: Arc<InnerDevice>,
}

//Swapchain Impl//
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
}

// Buffer Impl //
impl Device {
    pub fn create_buffer(&self, buffer_desc: &BufferDescription) -> BufferID {
        return self.inner.create_buffer(buffer_desc);
    }

    pub fn destroy_buffer(&self, id: BufferID) {
        self.inner.destroy_buffer(id.id);
    }
}

// Pipeline Manager //
impl Device {
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
