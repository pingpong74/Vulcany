use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

/// Represents the Vulkan API version used by the application.
/// Basically useless as only Vulkan 1.3 is used. Kept for future proofing
#[repr(u32)]
#[derive(Clone)]
pub enum ApiVersion {
    VkApi1_3 = ash::vk::API_VERSION_1_3,
}

/// High level abstraction for instance creation
/// Surface gets created along with the instance
pub struct InstanceDescription<W: HasDisplayHandle + HasWindowHandle> {
    pub api_version: ApiVersion,
    pub enable_validation_layers: bool,
    pub window: Arc<W>,
}

/// Very high level abstraction for device creation
/// Need to add more options
pub struct DeviceDescription {
    pub use_compute_queue: bool,
    pub use_transfer_queue: bool,
}

/// High level swapchain description
#[derive(Clone)]
pub struct SwapchainDescription {
    pub image_count: u32,
    pub width: u32,
    pub height: u32,
}
