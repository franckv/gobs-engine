use std::sync::Arc;

use gobs_core::{
    entity::{camera::Camera, light::Light},
    Transform,
};
use gobs_render::geometry::{Bounded, BoundingBox, Model};
use slotmap::{DefaultKey, Key};

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
    pub id: NodeId,
    pub value: NodeValue,
    pub(crate) transform: Transform,
    pub parent_transform: Transform,
    pub enabled: bool,
    pub(crate) parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub bounding_box: BoundingBox,
    pub updated: bool,
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

impl Node {
    pub(crate) fn new(
        value: NodeValue,
        transform: Transform,
        parent: Option<NodeId>,
        parent_transform: Transform,
    ) -> Self {
        let bounding_box = value.boundings();

        Self {
            id: NodeId::null(),
            value,
            transform,
            parent_transform,
            enabled: true,
            parent,
            children: Vec::new(),
            bounding_box,
            updated: true,
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
        self.updated = true;
    }

    pub fn reset_bounding_box(&mut self) {
        self.bounding_box = self.value.boundings();
    }

    pub fn global_transform(&self) -> Transform {
        let matrix = self.parent_transform * self.transform;

        matrix
    }

    pub(crate) fn set_parent_transform(&mut self, parent_transform: Transform) {
        self.parent_transform = parent_transform;
    }
}
