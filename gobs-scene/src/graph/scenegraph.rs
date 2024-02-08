use std::sync::Arc;

use slotmap::{DefaultKey, SlotMap};

use gobs_core::Transform;
use gobs_render::geometry::Model;

pub enum NodeValue {
    None,
    Model(Arc<Model>),
}

pub type NodeId = DefaultKey;

pub struct Node {
    pub value: NodeValue,
    pub transform: Transform,
    pub children: Vec<NodeId>,
}

impl Default for Node {
    fn default() -> Self {
        Self::new(NodeValue::None, Transform::IDENTITY)
    }
}

impl Node {
    pub fn new(value: NodeValue, transform: Transform) -> Self {
        Self {
            value,
            transform,
            children: Vec::new(),
        }
    }
}

pub struct SceneGraph {
    pub root: NodeId,
    arena: SlotMap<NodeId, Node>,
}

impl SceneGraph {
    pub fn new() -> Self {
        let mut arena = SlotMap::new();
        let root = arena.insert(Node::default());

        SceneGraph { root, arena }
    }

    pub fn get(&self, key: NodeId) -> Option<&Node> {
        self.arena.get(key)
    }

    pub fn get_mut(&mut self, key: NodeId) -> Option<&mut Node> {
        self.arena.get_mut(key)
    }

    pub fn insert(&mut self, parent: NodeId, node: Node) {
        let node = self
            .arena
            .contains_key(parent)
            .then_some(self.arena.insert(node));

        if let Some(parent) = self.arena.get_mut(parent) {
            if let Some(node) = node {
                parent.children.push(node);
            }
        }
    }

    pub fn visit<F>(&self, root: NodeId, f: &mut F)
    where
        F: FnMut(&Transform, &NodeValue),
    {
        if let Some(node) = self.arena.get(root) {
            for &child in &node.children {
                Self::visit(&self, child, f);
            }
            f(&node.transform, &node.value);
        }
    }
}
