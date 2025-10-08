use super::instance::Instance;
use std::sync::Arc;

use crate::{Fence, ImageID, ImageViewID, Semaphore, backend::swapchain::InnerSwapchain};

#[derive(Clone)]
pub struct Swapchain {
    pub(crate) inner: Arc<InnerSwapchain>,
}

impl Swapchain {
    pub fn acquire_image(
        &self,
        signal_semaphore: Option<&Semaphore>,
        signal_fence: Option<&Fence>,
    ) -> (ImageID, ImageViewID) {
        return self.inner.acquire_image(signal_semaphore, signal_fence);
    }

    pub fn preset(&self, wait_semaphore: &Semaphore) {
        self.inner.preset(wait_semaphore);
    }
}
