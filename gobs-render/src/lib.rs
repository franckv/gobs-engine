mod batch;
mod data;
mod model;
mod renderable;
mod renderer;
mod resources;

pub use gobs_render_graph::{GfxContext, RenderError, RenderFlags};
pub use gobs_render_hal::{
    AlignMode, Attribute, AttributeData, BlendMode, BufferType, CommandBuffer, CommandQueueType,
    CullMode, DynamicStateElem, FrontFace, Handle, ImageLayout, ObjectDataLayout, ObjectDataProp,
    Rect2D, RenderHAL, UniformData, UniformLayout, VertexAttribute, VertexData, Viewport,
};

pub use batch::RenderBatch;
pub use data::MaterialDataPropData;
pub use model::{Model, ModelId};
pub use renderable::Renderable;
pub use renderer::{Renderer, RendererOptions};

pub use resources::{
    Bounded, BoundingBox, GraphicsPipelineProperties, Material, MaterialData, MaterialInstance,
    MaterialInstanceLoader, MaterialInstanceProperties, MaterialLoader, MaterialProperties,
    MaterialsConfig, Mesh, MeshBuilder, MeshData, MeshGeometry, MeshLoader, MeshProperties,
    Pipeline, PipelineLoader, PipelineProperties, PipelinesConfig, Shapes, Texture, TextureData,
    TextureLoader, TexturePath, TextureProperties, TextureType, TextureUpdate,
};
