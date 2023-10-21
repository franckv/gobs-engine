use glam::{Vec3, Quat};
use uuid::Uuid;

use crate::model::Instance;

pub struct Node {
    transform: Instance,
    model: Uuid,
}

impl Node {
    pub fn new(position: Vec3, rotation: Quat, model: Uuid) -> Self {
        let transform = Instance {
            position, 
            rotation
        };

        Node { transform, model }
    }

    pub fn transform(&self) -> &Instance {
        &self.transform
    }

    pub fn model(&self) -> Uuid {
        self.model
    }
}
