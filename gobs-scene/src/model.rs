use std::sync::Arc;

use crate::mesh::MeshResource;

pub struct MeshData {
    pub offset: usize,
    pub len: usize,
}

pub struct Model {
    pub resource: Arc<MeshResource>,
    pub meshes: Vec<MeshData>,
}

impl Model {
    pub fn new(resource: Arc<MeshResource>) -> Self {
        Model {
            resource,
            meshes: Vec::new(),
        }
    }

    pub fn add_mesh(&mut self, offset: usize, len: usize) {
        self.meshes.push(MeshData { offset, len });
    }
}
