use std::sync::Arc;

use uuid::Uuid;

use gobs_core as core;
use gobs_material as material;

use core::geometry::mesh::Mesh;
use material::Material;

use crate::shader::Shader;

pub type ModelId = Uuid;

pub struct Model {
    pub id: ModelId,
    pub shader: Arc<Shader>,
    pub meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct ModelBuilder {
    meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder { meshes: Vec::new() }
    }

    pub fn meshes(mut self, meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>) -> Self {
        self.meshes = meshes;

        self
    }

    pub fn add_mesh(mut self, mesh: Arc<Mesh>, material: Option<Arc<Material>>) -> Self {
        self.meshes.push((mesh, material));

        self
    }

    pub fn build(self, shader: Arc<Shader>) -> Arc<Model> {
        Arc::new(Model {
            id: Uuid::new_v4(),
            shader,
            meshes: self.meshes,
        })
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
