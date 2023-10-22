use glam::{Quat, Vec3};
use uuid::Uuid;

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}

pub struct Node {
    transform: Transform,
    model: Uuid,
}

impl Node {
    pub fn new(position: Vec3, rotation: Quat, model: Uuid) -> Self {
        let transform = Transform { position, rotation };

        Node { transform, model }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn model(&self) -> Uuid {
        self.model
    }

    pub fn set_transform(&mut self, position: Vec3, rotation: Quat) {
        self.transform.position = position;
        self.transform.rotation = rotation;
    }
}
