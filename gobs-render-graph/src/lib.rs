mod context;
mod data;
mod error;
mod framedata;
mod graph;
mod job;
mod pass;
mod render_object;
mod resources;

use std::sync::Arc;

pub use context::GfxContext;
pub use data::{
    MaterialConstantData, MaterialDataLayout, MaterialDataProp, MaterialDataPropData,
    ObjectDataLayout, ObjectDataProp, SceneData, SceneDataLayout, SceneDataProp, TextureDataLayout,
    TextureDataProp, UniformBuffer, UniformData, UniformLayout, UniformPropData,
};
pub use error::RenderError;
pub use framedata::FrameData;
pub use graph::{FrameGraph, GraphConfig};
pub use job::RenderJob;
pub use pass::PassType;
pub use render_object::{MaterialId, MaterialInstanceId, MeshId, PassId, RenderObject};
pub use resources::{
    Bounded, BoundingBox, Material, MaterialData, MaterialInstance, MaterialInstanceLoader,
    MaterialInstanceProperties, MaterialLoader, MaterialProperties, MaterialsConfig, Mesh,
    MeshBuilder, MeshData, MeshGeometry, MeshLoader, MeshProperties, Pipeline, PipelineLoader,
    PipelinesConfig, Shapes, Texture, TextureData, TextureLoader, TexturePath, TextureProperties,
    TextureType, TextureUpdate,
};

pub type RenderPass = Arc<dyn pass::RenderPass>;
