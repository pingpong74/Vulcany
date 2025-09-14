use std::sync::Arc;

use crate::{
    RasterizationPipelineDescription,
    backend::pipelines::{InnerPipelineManager, InnerRasterizationPipeline},
};

pub struct PipelineManager {
    pub(crate) inner: Arc<InnerPipelineManager>,
}

impl PipelineManager {
    pub fn create_rasterization_pipeline(
        &self,
        raster_pipeline_desc: &RasterizationPipelineDescription,
    ) -> RasterizationPipeline {
        let (pipeline, layout) = self.inner.create_raster_pipeline_data(raster_pipeline_desc);

        return RasterizationPipeline {
            inner: Arc::new(InnerRasterizationPipeline {
                handle: pipeline,
                layout: layout,
                manager: self.inner.clone(),
            }),
        };
    }

    pub fn create_compute_pipeline() {}
}

pub struct RasterizationPipeline {
    inner: Arc<InnerRasterizationPipeline>,
}

pub struct ComputePipeline {}
