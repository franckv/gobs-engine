use gobs_core::Transform;

use crate::components::{BaseComponent, BoundingComponent, NodeId, NodeValue};

#[derive(Clone)]
pub struct Node {
    pub base: BaseComponent,
    pub bounding: BoundingComponent,
    pub(crate) transform: Transform,
    pub(crate) global_transform: Transform,
}

impl Default for Node {
    fn default() -> Self {
        let base = BaseComponent::default();
        let bounding = BoundingComponent::default();

        Self {
            base,
            bounding,
            transform: Transform::IDENTITY,
            global_transform: Transform::IDENTITY,
        }
    }
}

impl Node {
    pub(crate) fn new(
        value: NodeValue,
        transform: Transform,
        parent: Option<NodeId>,
        parent_transform: Transform,
    ) -> Self {
        let base = BaseComponent::new(value.clone(), parent);
        let bounding = BoundingComponent::new(value);

        Self {
            base,
            bounding,
            transform,
            global_transform: parent_transform * transform,
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn global_transform(&self) -> &Transform {
        &self.global_transform
    }

    pub fn update_transform<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Transform) -> bool,
    {
        self.base.updated |= f(&mut self.transform);
    }

    pub fn reset_bounding_box(&mut self, transform: Transform) {
        self.bounding.reset(&self.base.value);
        self.bounding.bounding_box = self.bounding.bounding_box.transform(transform);
    }
}
