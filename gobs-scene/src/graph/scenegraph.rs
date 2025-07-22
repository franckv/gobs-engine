use std::collections::{HashMap, hash_map::Entry};

use slotmap::SlotMap;

use gobs_core::Transform;
use gobs_render::{ModelId, RenderError};

use crate::{
    components::{BoundingComponent, NodeId, NodeValue},
    graph::node::Node,
};

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

    pub(crate) fn get_mut(&mut self, key: NodeId) -> Option<&mut Node> {
        self.arena.get_mut(key)
    }

    pub fn toggle(&mut self, key: NodeId) {
        if let Some(node) = self.get_mut(key) {
            node.base.enabled = !node.base.enabled;
        }
    }

    pub fn set_enabled(&mut self, key: NodeId, enabled: bool) {
        if let Some(node) = self.get_mut(key) {
            node.base.enabled = enabled;
        }
    }

    pub fn set_selected(&mut self, key: NodeId, selected: bool) {
        if let Some(node) = self.get_mut(key) {
            node.base.selected = selected;
        }
    }

    pub fn parent(&self, key: NodeId) -> Option<&Node> {
        self.get(key)
            .and_then(|node| node.base.parent)
            .and_then(|parent| self.get(parent))
    }

    pub fn update<F>(&mut self, key: NodeId, mut f: F)
    where
        F: FnMut(&mut Node) -> bool,
    {
        if let Some(node) = self.get_mut(key) {
            node.base.updated |= f(node);
        }
    }

    pub(crate) fn update_nodes(&mut self) {
        self.update_node(self.root, Transform::IDENTITY, false);
    }

    fn update_node(
        &mut self,
        key: NodeId,
        parent_transform: Transform,
        parent_updated: bool,
    ) -> bool {
        let mut children = vec![];
        let mut transform = parent_transform;
        let mut updated = false;

        if let Some(node) = self.get_mut(key) {
            updated = node.base.updated | parent_updated;
            node.base.updated = false;

            transform = parent_transform * node.transform;
            node.global_transform = transform;

            for child in &node.base.children {
                children.push(*child);
            }
        }

        let old_updated = updated;
        for child in &children {
            updated |= self.update_node(*child, transform, old_updated);
        }

        if updated {
            BoundingComponent::update(key, self);

            true
        } else {
            false
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
                let node = Node::new(value, transform, Some(parent), parent_node.global_transform);
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
        }

        node
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

    pub fn visit<F>(&self, root: NodeId, f: &mut F) -> Result<(), RenderError>
    where
        F: FnMut(&Node) -> Result<(), RenderError>,
    {
        self.visit_local(root, f)?;

        Ok(())
    }

    fn visit_local<F>(&self, root: NodeId, f: &mut F) -> Result<(), RenderError>
    where
        F: FnMut(&Node) -> Result<(), RenderError>,
    {
        if let Some(node) = self.get(root) {
            if node.base.enabled {
                for &child in &node.base.children {
                    self.visit_local(child, f)?;
                }
                f(node)?;
            }
        }

        Ok(())
    }

    pub fn visit_update<F>(&mut self, key: NodeId, f: &mut F)
    where
        F: FnMut(&mut Node) -> bool,
    {
        if let Some(node) = self.get(key) {
            for &child in &node.base.children.clone() {
                self.visit_update(child, f);
            }
        }

        self.update(key, f);
    }

    pub fn visit_sorted<F>(&self, root: NodeId, f: &mut F) -> Result<(), RenderError>
    where
        F: FnMut(&Transform, &NodeValue) -> Result<(), RenderError>,
    {
        let mut map: HashMap<ModelId, Vec<(Transform, NodeValue)>> = HashMap::new();

        self.visit(root, &mut |node| {
            if let NodeValue::Model(model) = &node.base.value {
                let transform = node.global_transform;
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
            Ok(())
        })?;

        for model_id in map.keys() {
            for (transform, node) in map.get(model_id).unwrap() {
                f(transform, node)?;
            }
        }

        Ok(())
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}
