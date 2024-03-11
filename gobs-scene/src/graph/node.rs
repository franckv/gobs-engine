use std::sync::Arc;

use gobs_core::{
    entity::{camera::Camera, light::Light},
    Transform,
};
use gobs_render::geometry::{Bounded, BoundingBox, Model};
use slotmap::DefaultKey;

#[derive(Clone, Debug)]
pub enum NodeValue {
    None,
    Model(Arc<Model>),
    Camera(Camera),
    Light(Light),
}

impl Bounded for NodeValue {
    fn boundings(&self) -> BoundingBox {
        match self {
            NodeValue::None => BoundingBox::default(),
            NodeValue::Model(model) => model.boundings(),
            NodeValue::Camera(_) => BoundingBox::default(),
            NodeValue::Light(_) => BoundingBox::default(),
        }
    }
}

pub type NodeId = DefaultKey;

#[derive(Clone)]
pub struct Node {
    pub value: NodeValue,
    pub transform: Transform,
    global_transform: Transform,
    pub enabled: bool,
    pub(crate) parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub bounding_box: BoundingBox,
}

impl Default for Node {
    fn default() -> Self {
        Self::new(
            NodeValue::None,
            Transform::IDENTITY,
            None,
            Transform::IDENTITY,
        )
    }
}

impl Bounded for Node {
    fn boundings(&self) -> BoundingBox {
        self.value.boundings().transform(self.global_transform)
    }
}

impl Node {
    pub(crate) fn new(
        value: NodeValue,
        transform: Transform,
        parent: Option<NodeId>,
        parent_transform: Transform,
    ) -> Self {
        let global_transform = parent_transform * transform;
        let bounding_box = value.boundings().transform(global_transform);

        Self {
            value,
            transform,
            global_transform,
            enabled: true,
            parent,
            children: Vec::new(),
            bounding_box,
        }
    }

    pub fn global_transform(&self) -> &Transform {
        &self.global_transform
    }

    pub(crate) fn set_global_transform(&mut self, global_transform: Transform) {
        self.global_transform = global_transform;
        self.bounding_box = self.boundings();
    }
}
