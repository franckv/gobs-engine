use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use slotmap::{DefaultKey, SlotMap};

use gobs_core::Transform;
use gobs_render::geometry::{Model, ModelId};

#[derive(Clone)]
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

    pub fn visit_sorted<F>(&self, root: NodeId, f: &mut F)
    where
        F: FnMut(&Transform, &NodeValue),
    {
        let mut map: HashMap<ModelId, Vec<(Transform, NodeValue)>> = HashMap::new();

        self.visit(root, &mut |transform, node| {
            if let NodeValue::Model(model) = node {
                match map.entry(model.id) {
                    Entry::Occupied(mut entry) => entry.get_mut().push((*transform, node.clone())),
                    Entry::Vacant(entry) => {
                        let values = vec![(*transform, node.clone())];
                        entry.insert(values);
                    }
                }
            }
        });

        for model_id in map.keys() {
            for (transform, node) in map.get(model_id).unwrap() {
                f(transform, node);
            }
        }
    }
}
