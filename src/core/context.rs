use crate::backend::{device::Device, instance::Instance};

use std::sync::Arc;

pub struct InstanceDescription {}

pub struct DeviceDescription {}

pub struct SwapchainDescription {}

pub struct Context {
    instance: Arc<backend::instance::Instance>,
}

impl Context {
    pub fn new() -> anyhow::Result<Context> {
        let instance = Instance::new();
    }
}
