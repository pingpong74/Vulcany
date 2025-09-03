use std::sync::Arc;

use crate::ImageViewDescription;
use crate::backend::image::*;

pub struct Image {
    pub(crate) inner: Arc<InnerImage>,
}

impl Image {
    pub fn create_image_view(&self, image_view_desc: &ImageViewDescription) -> ImageView {
        let view = self.inner.create_image_view_data(image_view_desc);

        return ImageView {
            inner: Arc::new(InnerImageView {
                handle: view,
                image: self.inner.clone(),
            }),
        };
    }
}

pub struct ImageView {
    pub(crate) inner: Arc<InnerImageView>,
}

pub struct Sampler {
    pub(crate) inner: Arc<InnerSampler>,
}
