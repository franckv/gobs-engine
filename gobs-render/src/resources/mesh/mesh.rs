use std::sync::Arc;

use gobs_gfx::GfxBuffer;
use gobs_resource::{
    geometry::{MeshGeometry, VertexAttribute},
    resource::{ResourceProperties, ResourceType},
};

use crate::resources::MeshLoader;

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub layer: u32,
}

impl ResourceProperties for MeshProperties {
    fn name(&self) -> &str {
        &self.name
    }
}

impl MeshProperties {
    pub fn with_geometry(geometry: Arc<MeshGeometry>, layer: u32) -> Self {
        Self {
            name: geometry.name.clone(),
            path: MeshPath::Mesh(geometry),
            layer,
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
