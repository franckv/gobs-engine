mod graph;
mod pass;
mod resources;

use std::sync::Arc;

pub use graph::{FrameGraph, GraphConfig};
pub use pass::{PassId, PassType};
pub use resources::{
    GraphicsPipelineProperties, Pipeline, PipelineLoader, PipelineProperties, PipelinesConfig,
};

pub type RenderPass = Arc<dyn pass::RenderPass>;
