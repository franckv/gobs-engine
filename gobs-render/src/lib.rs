mod batch;
mod builder;
mod config;
mod data;
mod job;
mod model;
mod pass;
mod render_object;
mod renderable;
mod renderer;
mod resources;

#[cfg(test)]
mod tests;

pub use gobs_render_graph::RenderError;
pub use gobs_render_hal::{
    AlignMode, Attribute, AttributeData, BlendMode, BufferType, CommandBuffer, CommandQueueType,
    CullMode, DynamicStateElem, FrontFace, GfxContext, Handle, ImageLayout, ObjectDataLayout,
    ObjectDataProp, Rect2D, RenderHAL, RenderHalConfig, UniformData, UniformLayout,
    VertexAttribute, VertexData, Viewport,
};
pub use gobs_render_material::{
    Material, MaterialDataPropData, MaterialInstance, MaterialInstanceLoader,
    MaterialInstanceProperties, MaterialLoader, MaterialsConfig, Pipeline, PipelineLoader,
    RenderFlags, Texture, TextureLoader, TextureProperties, TextureType, TextureUpdate,
};

pub use batch::RenderBatch;
pub use builder::{
    RenderBuilder, RenderMaterialBuilder, RenderMeshBuilder, RenderModelBuilder,
    RenderTextureBuilder, RenderType,
};
pub use config::RenderConfig;
pub use model::{Model, ModelId};
pub use renderable::Renderable;
pub use renderer::Renderer;

pub use resources::{
    Bounded, BoundingBox, Mesh, MeshBuilder, MeshData, MeshGeometry, MeshLoader, MeshProperties,
    ShapeBuilder, Shapes,
};
