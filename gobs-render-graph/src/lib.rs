mod framedata;
mod graph;
mod pass;

use std::sync::Arc;

pub use framedata::FrameData;
pub use graph::FrameGraph;
pub use pass::{PassId, PassType};

pub type RenderPass = Arc<dyn pass::RenderPass>;
