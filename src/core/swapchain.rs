use std::sync::Arc;

use crate::backend::swapchain::InnerSwapchain;

#[derive(Clone)]
pub struct Swapchain {
    pub(crate) inner: Arc<InnerSwapchain>,
}
