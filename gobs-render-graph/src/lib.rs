mod graph;
mod pass;

use std::sync::Arc;

pub use graph::{FrameGraph, GraphConfig};
pub use pass::PassType;

pub type RenderPass = Arc<dyn pass::RenderPass>;
