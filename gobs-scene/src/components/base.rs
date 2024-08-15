use std::sync::Arc;

use slotmap::{DefaultKey, Key};

use gobs_render::Model;
use gobs_resource::{
    entity::{camera::Camera, light::Light},
    geometry::{Bounded, BoundingBox},
};

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
pub struct BaseComponent {
    pub id: NodeId,
    pub value: NodeValue,
    pub enabled: bool,
    pub(crate) parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub updated: bool,
}

impl Default for BaseComponent {
    fn default() -> Self {
        Self {
            id: NodeId::null(),
            value: NodeValue::None,
            enabled: true,
            parent: None,
            children: Vec::new(),
            updated: true,
        }
    }
}

impl BaseComponent {
    pub fn new(value: NodeValue, parent: Option<NodeId>) -> Self {
        Self {
            id: NodeId::null(),
            value,
            enabled: true,
            parent,
            children: Vec::new(),
            updated: true,
        }
    }
}
