use gobs_resource::geometry::{Bounded, BoundingBox};

use super::NodeValue;

#[derive(Clone, Debug)]
pub struct BoundingComponent {
    pub bounding_box: BoundingBox,
}

impl BoundingComponent {
    pub fn new(value: NodeValue) -> Self {
        let bounding_box = match value {
            NodeValue::None => BoundingBox::default(),
            NodeValue::Model(model) => model.boundings(),
            NodeValue::Camera(_) => BoundingBox::default(),
            NodeValue::Light(_) => BoundingBox::default(),
        };

        Self { bounding_box }
    }

    pub fn reset(&mut self, value: &NodeValue) {
        self.bounding_box = match value {
            NodeValue::None => BoundingBox::default(),
            NodeValue::Model(model) => model.boundings(),
            NodeValue::Camera(_) => BoundingBox::default(),
            NodeValue::Light(_) => BoundingBox::default(),
        };
    }
}

impl Default for BoundingComponent {
    fn default() -> Self {
        Self {
            bounding_box: BoundingBox::default(),
        }
    }
}
