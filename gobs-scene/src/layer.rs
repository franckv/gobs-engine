use std::{collections::HashMap, sync::Arc};

use glam::{Quat, Vec3};

use gobs_render as render;

use render::{
    context::Gfx,
    graph::batch::BatchBuilder,
    model::{InstanceData, Model, ModelId},
};

use crate::data::Node;

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub nodes: Vec<Node<Arc<Model>>>,
    pub models: Vec<Arc<Model>>,
    pub instances: HashMap<ModelId, Vec<InstanceData>>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Layer {
            name: name.to_string(),
            visible: true,
            nodes: Vec::new(),
            models: Vec::new(),
            instances: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, position: Vec3, rotation: Quat, scale: Vec3, model: Arc<Model>) {
        let node = Node::new(position, rotation, scale, model);
        self.nodes.push(node);
    }

    pub fn update(&mut self, _gfx: &Gfx) {
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

            let instance_data = InstanceData::new(node.model().shader.instance_flags)
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
    }

    pub fn render<'a>(&'a self, mut batch: BatchBuilder<'a>) -> BatchBuilder<'a> {
        for model in &self.models {
            let instances = self.instances.get(&model.id).unwrap();

            batch = batch.draw_indexed(model.clone(), instances);
        }

        batch
    }
}
