mod context;
mod error;
mod graph;
mod pass;
mod render_object;
mod uniform;

use std::sync::Arc;

pub use gobs_gfx::{BlendMode, CullMode, Display, ImageUsage};

pub use context::GfxContext;
pub use error::RenderError;
pub use graph::FrameGraph;
pub use pass::{PassId, PassType};
pub use render_object::RenderObject;
pub use uniform::UniformBuffer;

pub type RenderPass = Arc<dyn pass::RenderPass>;
