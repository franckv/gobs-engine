use std::collections::{hash_map::Entry, HashMap};

use slotmap::SlotMap;

use gobs_core::Transform;
use gobs_render::ModelId;
use gobs_resource::geometry::BoundingBox;

use super::node::Node;
use crate::components::{NodeId, NodeValue};

#[derive(Clone)]
pub struct SceneGraph {
    pub root: NodeId,
    arena: SlotMap<NodeId, Node>,
}

impl SceneGraph {
    pub fn new() -> Self {
        let mut arena = SlotMap::new();
        let root_id = arena.insert(Node::default());
        if let Some(root) = arena.get_mut(root_id) {
            root.base.id = root_id;
        }

        SceneGraph {
            root: root_id,
            arena,
        }
    }

    pub fn get(&self, key: NodeId) -> Option<&Node> {
        self.arena.get(key)
    }

    pub fn get_mut(&mut self, key: NodeId) -> Option<&mut Node> {
        self.arena.get_mut(key)
    }

    pub fn toggle(&mut self, key: NodeId) {
        if let Some(node) = self.get_mut(key) {
            node.base.enabled = !node.base.enabled;
        }
    }

    pub fn parent(&self, key: NodeId) -> Option<&Node> {
        self.get(key)
            .and_then(|node| node.base.parent)
            .and_then(|parent| self.get(parent))
    }

    pub fn parent_transform(&self, key: NodeId) -> Transform {
        match self.parent(key) {
            Some(parent) => parent.global_transform(),
            None => Transform::IDENTITY,
        }
    }

    pub fn update<F>(&mut self, key: NodeId, mut f: F)
    where
        F: FnMut(&mut Node),
    {
        let mut updated = false;

        if let Some(node) = self.get_mut(key) {
            f(node);
            updated = node.base.updated;
            node.base.updated = false;
        }

        if updated {
            let parent_transform = self.parent_transform(key);
            self.update_global_transform(key, parent_transform);
            self.update_bounding_box(key);
        }
    }

    fn update_global_transform(&mut self, key: NodeId, parent_transform: Transform) {
        let mut children = vec![];
        let mut global_transform = Transform::IDENTITY;

        if let Some(node) = self.get_mut(key) {
            node.set_parent_transform(parent_transform);
            global_transform = node.global_transform();

            for &child in &node.base.children {
                children.push(child);
            }
        }

        for child in children {
            self.update_global_transform(child, global_transform);
        }
    }

    pub fn remove(&mut self, key: NodeId) -> Option<Node> {
        self.arena.remove(key)
    }

    pub fn set_root(&mut self, value: NodeValue, transform: Transform) -> NodeId {
        self.arena.clear();
        let root = Node::new(value, transform, None, Transform::IDENTITY);
        let root_id = self.arena.insert(root);
        self.root = root_id;

        if let Some(root) = self.get_mut(root_id) {
            root.base.id = root_id;
        }

        self.root
    }

    pub fn insert(
        &mut self,
        parent: NodeId,
        value: NodeValue,
        transform: Transform,
    ) -> Option<NodeId> {
        let node = match self.get(parent) {
            Some(parent_node) => {
                let node = Node::new(
                    value,
                    transform,
                    Some(parent),
                    parent_node.global_transform(),
                );
                let node_id = self.arena.insert(node);
                if let Some(node) = self.get_mut(node_id) {
                    node.base.id = node_id;
                }
                Some(node_id)
            }
            None => None,
        };

        if let Some(parent_node) = self.get_mut(parent) {
            if let Some(node) = node {
                parent_node.base.children.push(node);
            }
            self.update_bounding_box(parent);
        }

        node
    }

    fn bounding_box(&self, key: NodeId) -> BoundingBox {
        match self.get(key) {
            Some(node) => node.bounding.bounding_box,
            None => BoundingBox::default(),
        }
    }

    pub fn update_bounding_box(&mut self, key: NodeId) {
        if let Some(node) = self.get_mut(key) {
            node.reset_bounding_box();
        }

        let mut bb = self.bounding_box(key);

        if let Some(node) = self.get(key) {
            for &child in &node.base.children {
                if let Some(child) = self.get(child) {
                    let child_bb = child.bounding.bounding_box.transform(child.transform);
                    bb.extends_box(child_bb);
                }
            }
        }

        if let Some(node) = self.get_mut(key) {
            node.bounding.bounding_box = bb;
        }

        if let Some(node) = self.get(key) {
            if let Some(parent) = node.base.parent {
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
            let node = self.insert(
                local_root,
                target_node.base.value.clone(),
                target_node.transform,
            );
            if let Some(node) = node {
                for &child in &target_node.base.children {
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
            if node.base.enabled {
                for &child in &node.base.children {
                    self.visit_local(child, f);
                }
                f(&node);
            }
        }
    }

    pub fn visit_update<F>(&mut self, key: NodeId, f: &mut F)
    where
        F: FnMut(&mut Node),
    {
        if let Some(node) = self.get(key) {
            for &child in &node.base.children.clone() {
                self.visit_update(child, f);
            }
        }

        self.update(key, f);
    }

    pub fn visit_sorted<F>(&self, root: NodeId, f: &mut F)
    where
        F: FnMut(&Transform, &NodeValue),
    {
        let mut map: HashMap<ModelId, Vec<(Transform, NodeValue)>> = HashMap::new();

        self.visit(root, &mut |node| {
            if let NodeValue::Model(model) = &node.base.value {
                let transform = node.global_transform();
                match map.entry(model.id) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push((transform, node.base.value.clone()))
                    }
                    Entry::Vacant(entry) => {
                        let values = vec![(transform, node.base.value.clone())];
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
