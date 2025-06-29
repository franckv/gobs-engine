use gobs_resource::geometry::{Bounded, BoundingBox};

use crate::{
    components::{NodeId, NodeValue},
    graph::scenegraph::SceneGraph,
};

#[derive(Clone, Debug, Default)]
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

    pub fn update(key: NodeId, graph: &mut SceneGraph) {
        let mut bb = BoundingBox::default();

        if let Some(node) = graph.get_mut(key) {
            node.bounding.reset(&node.base.value);
            node.bounding.bounding_box =
                node.bounding.bounding_box.transform(node.global_transform);
            bb = node.bounding.bounding_box;
        }

        if let Some(node) = graph.get(key) {
            for child in &node.base.children {
                if let Some(child) = graph.get(*child) {
                    let child_bb = child.bounding.bounding_box;
                    bb.extends_box(child_bb);
                }
            }
        }

        if let Some(node) = graph.get_mut(key) {
            node.bounding.bounding_box = bb;
        }
    }

    fn reset(&mut self, value: &NodeValue) {
        self.bounding_box = match value {
            NodeValue::None => BoundingBox::default(),
            NodeValue::Model(model) => model.boundings(),
            NodeValue::Camera(_) => BoundingBox::default(),
            NodeValue::Light(_) => BoundingBox::default(),
        };
    }
}
