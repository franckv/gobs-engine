use crate::model::Instance;

pub struct Node {
    transform: Instance,
    model: usize
}

impl Node {
    pub fn new(transform: Instance, model: usize) -> Self {
        Node {
            transform,
            model
        }
    }

    pub fn transform(&self) -> &Instance {
        &self.transform
    }

    pub fn model(&self) -> usize {
        self.model
    }
}
