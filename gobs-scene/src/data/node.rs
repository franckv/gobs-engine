use glam::{Quat, Vec3};

use crate::transform::Transform;

pub struct Node<D> {
    transform: Transform,
    model: D,
}

impl<D> Node<D>
where
    D: Clone,
{
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3, model: D) -> Self {
        let transform = Transform {
            translation,
            rotation,
            scale,
        };

        Node { transform, model }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn model(&self) -> D {
        self.model.clone()
    }

    pub fn set_transform(&mut self, translation: Vec3, rotation: Quat, scale: Vec3) {
        self.transform.translation = translation;
        self.transform.rotation = rotation;
        self.transform.scale = scale;
    }
}
