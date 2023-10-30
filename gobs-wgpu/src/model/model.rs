use std::sync::Arc;

use glam::Vec3;
use uuid::Uuid;

use crate::model::{Material, Mesh};

pub struct ModelBuilder {
    scale: Vec3,
    meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder {
            scale: Vec3::splat(1.),
            meshes: Vec::new(),
        }
    }

    pub fn scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;

        self
    }

    pub fn add_mesh(mut self, mesh: Arc<Mesh>, material: Option<Arc<Material>>) -> Self {
        self.meshes.push((mesh, material));

        self
    }

    pub fn build(self) -> Arc<Model> {
        Arc::new(Model {
            id: Uuid::new_v4(),
            scale: self.scale,
            meshes: self.meshes,
        })
    }
}

pub struct Model {
    pub id: Uuid,
    pub scale: Vec3,
    pub meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}
