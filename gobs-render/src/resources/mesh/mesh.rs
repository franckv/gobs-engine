use std::sync::Arc;

use gobs_render_hal::{Handle, RenderHAL, VertexAttribute};
use gobs_resource::{ResourceProperties, ResourceType};

use crate::resources::{MeshGeometry, MeshLoader};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mesh;

impl ResourceType for Mesh {
    type ResourceData = MeshData;
    type ResourceBackend<'a> = dyn RenderHAL + 'a;
    type ResourceProperties = MeshProperties;
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
    pub vertex_attributes: VertexAttribute,
    pub layer: u32,
}

impl ResourceProperties for MeshProperties {
    fn name(&self) -> &str {
        &self.name
    }
}

impl MeshProperties {
    pub fn with_geometry(
        geometry: Arc<MeshGeometry>,
        vertex_attributes: VertexAttribute,
        layer: u32,
    ) -> Self {
        Self {
            name: geometry.name.clone(),
            path: MeshPath::Mesh(geometry),
            vertex_attributes,
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
    pub vertex_view: Handle,
    pub index_view: Handle,
    pub index_len: usize,
}
