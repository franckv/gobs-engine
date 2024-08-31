mod batch;
mod context;
mod graph;
mod material;
mod model;
mod pass;
mod renderable;
mod resources;
mod stats;

use std::sync::Arc;

pub use gobs_gfx::{BlendMode, CullMode, Display, ImageUsage};

pub use batch::RenderBatch;
pub use context::Context;
pub use graph::{FrameGraph, RenderError};
pub use material::{Material, MaterialInstance, MaterialProperty};
pub use model::{Model, ModelId};
pub use pass::PassType;
pub use renderable::{Renderable, RenderableLifetime};

pub type RenderPass = Arc<dyn pass::RenderPass>;
