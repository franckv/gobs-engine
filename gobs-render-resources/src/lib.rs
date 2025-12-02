mod resources;

pub use resources::{
    Bounded, BoundingBox, Material, MaterialData, MaterialInstance, MaterialInstanceLoader,
    MaterialInstanceProperties, MaterialLoader, MaterialProperties, MaterialsConfig, Mesh,
    MeshBuilder, MeshData, MeshGeometry, MeshLoader, MeshProperties, Pipeline, PipelineLoader,
    PipelinesConfig, Shapes, Texture, TextureData, TextureLoader, TexturePath, TextureProperties,
    TextureType, TextureUpdate,
};

pub use gobs_gfx::{VertexAttribute, VertexData};
