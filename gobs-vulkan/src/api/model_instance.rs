use std::sync::Arc;

use api::model::Model;

pub struct ModelInstance<V, T> {
    model: Arc<Model<V>>,
    transform: T
}

impl<V: Copy, T: Copy> ModelInstance<V, T> {
    pub fn new(model: Arc<Model<V>>, transform: T) -> Self {
        ModelInstance {
            model,
            transform
        }
    }

    pub fn model(&self) -> &Arc<Model<V>> {
        &self.model
    }

    pub fn transform(&self) -> T {
        self.transform
    }
}