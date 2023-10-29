use std::sync::Arc;

use glam::{Quat, Vec3};
use gobs_wgpu::model::Model;

use crate::transform::Transform;

pub struct Node {
    transform: Transform,
    model: Arc<Model>,
}

impl Node {
    pub fn new(translation: Vec3, rotation: Quat, model: Arc<Model>) -> Self {
        let transform = Transform {
            translation,
            rotation,
        };

        Node { transform, model }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn model(&self) -> Arc<Model> {
        self.model.clone()
    }

    pub fn set_transform(&mut self, translation: Vec3, rotation: Quat) {
        self.transform.translation = translation;
        self.transform.rotation = rotation;
    }
}
