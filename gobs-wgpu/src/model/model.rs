use std::sync::Arc;

use uuid::Uuid;

use crate::model::{Material, Mesh};

pub struct ModelBuilder {
    scale: f32,
    meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder {
            scale: 1.,
            meshes: Vec::new(),
        }
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;

        self
    }

    pub fn add_mesh(mut self, mesh: Arc<Mesh>, material: Option<Arc<Material>>) -> Self {
        self.meshes.push((mesh, material));

        self
    }

    pub fn build(self) -> Model {
        Model {
            id: Uuid::new_v4(),
            scale: self.scale,
            meshes: self.meshes,
        }
    }
}

pub struct Model {
    pub id: Uuid,
    pub scale: f32,
    pub meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}
