use gobs_core::Transform;

use crate::components::{BaseComponent, BoundingComponent, NodeId, NodeValue};

#[derive(Clone)]
pub struct Node {
    pub base: BaseComponent,
    pub bounding: BoundingComponent,
    pub(crate) transform: Transform,
    pub parent_transform: Transform,
}

impl Default for Node {
    fn default() -> Self {
        let base = BaseComponent::default();
        let bounding = BoundingComponent::default();

        Self {
            base,
            bounding,
            transform: Transform::IDENTITY,
            parent_transform: Transform::IDENTITY,
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
            parent_transform,
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn update_transform<F>(&mut self, f: F)
    where
        F: Fn(&mut Transform),
    {
        f(&mut self.transform);
        self.base.updated = true;
    }

    pub fn reset_bounding_box(&mut self) {
        self.bounding.reset(&self.base.value);
    }

    pub fn global_transform(&self) -> Transform {
        self.parent_transform * self.transform
    }

    pub(crate) fn set_parent_transform(&mut self, parent_transform: Transform) {
        self.parent_transform = parent_transform;
    }
}
