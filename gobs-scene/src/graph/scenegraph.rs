use std::collections::{hash_map::Entry, HashMap};

use slotmap::SlotMap;

use gobs_core::Transform;
use gobs_render::geometry::ModelId;

use super::node::{Node, NodeId, NodeValue};

#[derive(Clone)]
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

    pub fn toggle(&mut self, key: NodeId) {
        if let Some(node) = self.get_mut(key) {
            node.enabled = !node.enabled;
        }
    }

    pub fn parent(&self, key: NodeId) -> Option<&Node> {
        self.get(key)
            .and_then(|node| node.parent)
            .and_then(|parent| self.get(parent))
    }

    pub fn update<F>(&mut self, key: NodeId, mut f: F)
    where
        F: FnMut(&mut Transform, &mut NodeValue),
    {
        if let Some(node) = self.get_mut(key) {
            f(&mut node.transform, &mut node.value);
        }

        if let Some(node) = self.get(key) {
            if let Some(parent) = self.parent(key) {
                let parent_key = node.parent.unwrap();
                self.update_transform(key, *parent.global_transform());
                self.update_bounding_box(parent_key);
            } else {
                self.update_transform(key, Transform::IDENTITY);
            }
        }
    }

    fn update_transform(&mut self, key: NodeId, parent_transform: Transform) {
        let mut children = vec![];
        let mut global_transform = Transform::IDENTITY;

        if let Some(node) = self.get_mut(key) {
            node.apply_parent_transform(parent_transform);
            global_transform = *node.global_transform();

            for &child in &node.children {
                children.push(child);
            }
        }

        for child in children {
            self.update_transform(child, global_transform);
        }
    }

    pub fn remove(&mut self, key: NodeId) -> Option<Node> {
        self.arena.remove(key)
    }

    pub fn set_root(&mut self, value: NodeValue, transform: Transform) -> NodeId {
        self.arena.clear();
        self.root = self
            .arena
            .insert(Node::new(value, transform, None, Transform::IDENTITY));

        self.root
    }

    pub fn insert(
        &mut self,
        parent: NodeId,
        value: NodeValue,
        transform: Transform,
    ) -> Option<NodeId> {
        let node = match self.get(parent) {
            Some(parent_node) => Some(self.arena.insert(Node::new(
                value,
                transform,
                Some(parent),
                *parent_node.global_transform(),
            ))),
            None => None,
        };

        if let Some(parent_node) = self.get_mut(parent) {
            if let Some(node) = node {
                parent_node.children.push(node);
            }
            self.update_bounding_box(parent);
        }

        node
    }

    pub fn update_bounding_box(&mut self, key: NodeId) {
        let bb = match self.get(key) {
            Some(node) => {
                let mut bb = node.bounding_box;
                for &child in &node.children {
                    if let Some(child) = self.get(child) {
                        bb.extends_box(child.bounding_box);
                    }
                }

                bb
            }
            None => return,
        };

        if let Some(node) = self.get_mut(key) {
            node.bounding_box = bb;
            if let Some(parent) = node.parent {
                self.update_bounding_box(parent);
            }
        }
    }

    pub fn insert_subgraph(
        &mut self,
        local_root: NodeId,
        target_root: NodeId,
        subgraph: &SceneGraph,
    ) -> Option<NodeId> {
        if let Some(target_node) = subgraph.get(target_root) {
            let node = self.insert(local_root, target_node.value.clone(), target_node.transform);
            if let Some(node) = node {
                for &child in &target_node.children {
                    self.insert_subgraph(node, child, subgraph);
                }

                return Some(node);
            }
        }

        None
    }

    pub fn visit<F>(&self, root: NodeId, f: &mut F)
    where
        F: FnMut(&Node),
    {
        self.visit_local(root, f);
    }

    fn visit_local<F>(&self, root: NodeId, f: &mut F)
    where
        F: FnMut(&Node),
    {
        if let Some(node) = self.get(root) {
            if node.enabled {
                for &child in &node.children {
                    self.visit_local(child, f);
                }
                f(&node);
            }
        }
    }

    pub fn visit_update<F>(&mut self, key: NodeId, f: &mut F)
    where
        F: FnMut(&mut Transform, &NodeValue),
    {
        if let Some(node) = self.get(key) {
            if let Some(parent) = self.parent(key) {
                let parent_key = node.parent.unwrap();
                self.visit_update_local(key, *parent.global_transform(), f);
                self.update_bounding_box(parent_key);
            } else {
                self.visit_update_local(key, Transform::IDENTITY, f);
            }
        }
    }

    fn visit_update_local<F>(&mut self, key: NodeId, parent_transform: Transform, f: &mut F)
    where
        F: FnMut(&mut Transform, &NodeValue),
    {
        if let Some(node) = self.get_mut(key) {
            f(&mut node.transform, &node.value);
            node.apply_parent_transform(parent_transform);
        }
        if let Some(node) = self.get(key) {
            let global_transform = *node.global_transform();
            for &child in &node.children.clone() {
                self.visit_update_local(child, global_transform, f);
            }
        }
    }

    pub fn visit_sorted<F>(&self, root: NodeId, f: &mut F)
    where
        F: FnMut(&Transform, &NodeValue),
    {
        let mut map: HashMap<ModelId, Vec<(Transform, NodeValue)>> = HashMap::new();

        self.visit(root, &mut |node| {
            if let NodeValue::Model(model) = &node.value {
                let transform = node.global_transform();
                match map.entry(model.id) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push((*transform, node.value.clone()))
                    }
                    Entry::Vacant(entry) => {
                        let values = vec![(*transform, node.value.clone())];
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
