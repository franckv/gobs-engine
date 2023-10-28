use std::sync::Arc;

use glam::{Quat, Vec3};
use gobs_wgpu::model::Model;

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}

pub struct Node {
    transform: Transform,
    model: Arc<Model>,
}

impl Node {
    pub fn new(position: Vec3, rotation: Quat, model: Arc<Model>) -> Self {
        let transform = Transform { position, rotation };

        Node { transform, model }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn model(&self) -> Arc<Model> {
        self.model.clone()
    }

    pub fn set_transform(&mut self, position: Vec3, rotation: Quat) {
        self.transform.position = position;
        self.transform.rotation = rotation;
    }
}
