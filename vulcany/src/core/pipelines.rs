use std::sync::Arc;

use crate::{
    ComputePipelineDescription, RasterizationPipelineDescription,
    backend::pipelines::{InnerComputePipeline, InnerPipelineManager, InnerRasterizationPipeline},
};

pub struct PipelineManager {
    pub(crate) inner: Arc<InnerPipelineManager>,
}

impl PipelineManager {
    pub fn create_rasterization_pipeline(&self, raster_pipeline_desc: &RasterizationPipelineDescription) -> Pipeline {
        let (pipeline, layout) = self.inner.create_raster_pipeline_data(raster_pipeline_desc);

        return Pipeline::RasterizationPipeline(Arc::new(InnerRasterizationPipeline {
            handle: pipeline,
            layout: layout,
            manager: self.inner.clone(),
        }));
    }

    pub fn create_compute_pipeline(&self, compute_pipeline_desc: &ComputePipelineDescription) -> Pipeline {
        unimplemented!()
    }
}

pub enum Pipeline {
    RasterizationPipeline(Arc<InnerRasterizationPipeline>),
    ComputePipeline(Arc<InnerComputePipeline>),
}

impl Pipeline {
    pub(crate) fn get_handle(&self) -> ash::vk::Pipeline {
        match self {
            Pipeline::RasterizationPipeline(inner) => inner.handle,
            Pipeline::ComputePipeline(inner) => inner.handle,
        }
    }

    pub(crate) fn get_layout(&self) -> ash::vk::PipelineLayout {
        match self {
            Pipeline::RasterizationPipeline(inner) => inner.layout,
            Pipeline::ComputePipeline(inner) => inner.layout,
        }
    }
}
