mod context;
mod error;
mod framedata;
mod graph;
mod job;
mod pass;
mod render_object;
mod resources;

use std::sync::Arc;

pub use context::GfxContext;
pub use error::RenderError;
pub use framedata::FrameData;
pub use graph::{FrameGraph, GraphConfig};
pub use job::RenderJob;
pub use pass::PassType;
pub use render_object::{
    MaterialId, MaterialInstanceId, MeshId, PassId, RenderFlags, RenderObject,
};
pub use resources::{
    Bounded, BoundingBox, GraphicsPipelineProperties, Material, MaterialData, MaterialInstance,
    MaterialInstanceLoader, MaterialInstanceProperties, MaterialLoader, MaterialProperties,
    MaterialsConfig, Mesh, MeshBuilder, MeshData, MeshGeometry, MeshLoader, MeshProperties,
    Pipeline, PipelineLoader, PipelineProperties, PipelinesConfig, Shapes, Texture, TextureData,
    TextureLoader, TexturePath, TextureProperties, TextureType, TextureUpdate,
};

pub type RenderPass = Arc<dyn pass::RenderPass>;
