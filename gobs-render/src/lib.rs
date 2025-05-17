mod batch;
mod context;
mod error;
mod graph;
mod manager;
mod materials;
mod model;
mod pass;
mod renderable;
mod resources;
mod stats;

use std::sync::Arc;

pub use gobs_gfx::{BlendMode, CullMode, Display, ImageUsage};

pub use batch::RenderBatch;
pub use context::GfxContext;
pub use error::RenderError;
pub use graph::FrameGraph;
pub use materials::{Material, MaterialInstance, MaterialProperty};
pub use model::{Model, ModelId};
pub use pass::PassType;
pub use renderable::{Renderable, RenderableLifetime};
pub use resources::{
    Mesh, MeshData, MeshLoader, Texture, TextureData, TextureLoader, TexturePath,
    TextureProperties, TextureType, TextureUpdate,
};

pub type RenderPass = Arc<dyn pass::RenderPass>;
