use glam::{Quat, Vec3};
use uuid::Uuid;

use crate::model::Instance;

pub struct Node {
    transform: Instance,
    model: Uuid,
}

impl Node {
    pub fn new(position: Vec3, rotation: Quat, model: Uuid) -> Self {
        let transform = Instance { position, rotation };

        Node { transform, model }
    }

    pub fn transform(&self) -> &Instance {
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
