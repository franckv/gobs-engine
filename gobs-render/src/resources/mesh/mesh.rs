use std::sync::Arc;

use gobs_graphics::{MeshGeometry, VertexAttribute};
use gobs_render_hal::{Handle, RenderHAL};
use gobs_resource::{ResourceProperties, ResourceType};

use crate::resources::MeshLoader;

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
    Bytes((Vec<u8>, Vec<u32>)),
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
        name: &str,
        geometry: Arc<MeshGeometry>,
        vertex_attributes: VertexAttribute,
        layer: u32,
    ) -> Self {
        Self {
            name: name.to_string(),
            path: MeshPath::Mesh(geometry),
            vertex_attributes,
            layer,
        }
    }

    pub fn with_bytes(
        name: &str,
        bytes: Vec<u8>,
        indices: Vec<u32>,
        vertex_attributes: VertexAttribute,
        layer: u32,
    ) -> Self {
        Self {
            name: name.to_string(),
            path: MeshPath::Bytes((bytes, indices)),
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
