mod context;
mod data;
mod error;
mod framedata;
mod graph;
mod job;
mod pass;
mod render_object;

use std::sync::Arc;

pub use context::GfxContext;
pub use data::SceneData;
pub use error::RenderError;
pub use framedata::FrameData;
pub use graph::{FrameGraph, GraphConfig};
pub use job::RenderJob;
pub use render_object::{
    MaterialId, MaterialInstanceId, MeshId, PassId, RenderFlags, RenderObject,
};

pub type RenderPass = Arc<dyn pass::RenderPass>;
