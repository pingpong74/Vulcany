use std::sync::Arc;

use crate::{RasterizationPipelineDescription, backend::pipelines::InnerPipelineManager};

pub struct PipelineManager {
    pub(crate) inner: Arc<InnerPipelineManager>,
}

impl PipelineManager {
    pub fn create_rasterization_pipeline(
        &self,
        raster_pipeline_desc: &RasterizationPipelineDescription,
    ) {
        let _ = self.inner.create_raster_pipeline_data(raster_pipeline_desc);
    }

    pub fn create_compute_pipeline() {}
}

pub struct RasterizationPipeline {}

pub struct ComputePipeline {}
