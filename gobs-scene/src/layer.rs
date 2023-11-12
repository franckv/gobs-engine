use std::{collections::HashMap, sync::Arc};

use glam::{Quat, Vec3};

use gobs_render as render;

use render::{
    context::Gfx,
    graph::batch::BatchBuilder,
    model::{InstanceData, Model, ModelId},
};

use crate::data::Node;

type LayerNode = Node<Arc<Model>>;

pub struct Layer {
    pub name: String,
    nodes: Vec<LayerNode>,
    visible: bool,
    dirty: bool,
    models: Vec<Arc<Model>>,
    instances: HashMap<ModelId, Vec<InstanceData>>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Layer {
            name: name.to_string(),
            visible: true,
            dirty: true,
            nodes: Vec::new(),
            models: Vec::new(),
            instances: HashMap::new(),
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible
    }

    pub fn add_node(&mut self, position: Vec3, rotation: Quat, scale: Vec3, model: Arc<Model>) {
        let node = Node::new(position, rotation, scale, model);
        self.nodes.push(node);
        self.dirty = true
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<LayerNode> {
        self.dirty = true;
        &mut self.nodes
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        self.dirty = true;
    }

    pub fn update(&mut self, _gfx: &Gfx) {
        if !self.dirty {
            return;
        }

        self.models.clear();
        self.instances.clear();

        for node in &self.nodes {
            let model_id = node.model().id;

            if !self.models.contains(&node.model()) {
                self.models.push(node.model().clone());
            }

            if !self.instances.contains_key(&model_id) {
                self.instances.insert(model_id, Vec::new());
            }

            let instance_data = InstanceData::new()
                .model_transform(
                    node.transform().translation,
                    node.transform().rotation,
                    node.transform().scale,
                )
                .normal_rot(node.transform().rotation)
                .build();

            self.instances
                .get_mut(&model_id)
                .unwrap()
                .push(instance_data);
        }

        self.dirty = false
    }

    pub fn render<'a>(&'a self, mut batch: BatchBuilder<'a>) -> BatchBuilder<'a> {
        for model in &self.models {
            let instances = self.instances.get(&model.id).unwrap();

            batch = batch.draw_indexed(model.clone(), instances);
        }

        batch
    }
}
