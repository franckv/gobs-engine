use std::sync::Arc;

use gobs_gfx::GfxBuffer;
use gobs_resource::{
    geometry::{MeshGeometry, VertexAttribute},
    resource::ResourceType,
};

use crate::resources::MeshLoader;

#[derive(Clone, Copy, Debug)]
pub struct Mesh;

impl ResourceType for Mesh {
    type ResourceData = MeshData;
    type ResourceProperties = MeshProperties;
    type ResourceParameter = VertexAttribute;
    type ResourceLoader = MeshLoader;
}

#[derive(Clone, Debug)]
pub enum MeshPath {
    Default,
    File(String),
    Mesh(Arc<MeshGeometry>),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct MeshProperties {
    pub name: String,
    pub path: MeshPath,
}

impl MeshProperties {
    pub fn with_geometry(name: &str, geometry: Arc<MeshGeometry>) -> Self {
        Self {
            name: name.to_string(),
            path: MeshPath::Mesh(geometry),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MeshPrimitiveType {
    Triangle,
}

#[derive(Clone)]
pub struct MeshData {
    pub ty: MeshPrimitiveType,
    pub vertex_buffer: Arc<GfxBuffer>,
    pub index_buffer: Arc<GfxBuffer>,
    pub vertices_offset: u64,
    pub vertices_len: usize,
    pub vertices_count: usize,
    pub indices_offset: usize,
    pub indices_len: usize,
}

impl MeshData {
    // pub fn new() -> Self {
    //     Self {
    //         ty: MeshPrimitiveType::Triangle,
    //         material,
    //         material_binding,
    //         vertex_buffer: vertex_buffer.clone(),
    //         index_buffer: index_buffer.clone(),
    //         vertices_offset,
    //         indices_offset,
    //         indices_len,
    //     }
    // }
}
