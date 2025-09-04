use std::sync::Arc;

use crate::backend::pipelines::InnerPipelineManager;

pub struct PipelineManager {
    inner: Arc<InnerPipelineManager>,
}

impl PipelineManager {
    pub fn create_rasterization_pipeline() {}

    pub fn create_compute_pipeline() {}
}

pub struct RasterizationPipeline {}

pub struct ComputePipeline {}
