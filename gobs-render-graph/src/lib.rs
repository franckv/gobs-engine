mod context;
mod data;
mod error;
mod framedata;
mod graph;
mod job;
mod pass;
mod render_object;

use std::sync::Arc;

pub use gobs_gfx::{BlendMode, CullMode, Display, ImageUsage};

pub use context::GfxContext;
pub use data::{SceneData, UniformBuffer, UniformPropData};
pub use error::RenderError;
pub use framedata::FrameData;
pub use graph::FrameGraph;
pub use pass::{PassId, PassType};
pub use render_object::RenderObject;

pub type RenderPass = Arc<dyn pass::RenderPass>;
