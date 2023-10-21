use uuid::Uuid;

use crate::model::Instance;

pub struct Node {
    transform: Instance,
    model: Uuid,
}

impl Node {
    pub fn new(transform: Instance, model: Uuid) -> Self {
        Node { transform, model }
    }

    pub fn transform(&self) -> &Instance {
        &self.transform
    }

    pub fn model(&self) -> Uuid {
        self.model
    }
}
