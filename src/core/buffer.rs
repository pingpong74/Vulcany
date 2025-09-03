use ash::vk;
use std::sync::Arc;

use crate::backend::buffer::InnerBuffer;

pub struct Buffer {
    pub(crate) inner: Arc<InnerBuffer>,
}
