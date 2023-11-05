use std::sync::Arc;

use glam::{Quat, Vec3};

use gobs_wgpu as render;

use render::{
    model::{InstanceData, Model, ModelInstance},
    render::{BatchBuilder, Gfx},
};

use crate::data::Node;

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub nodes: Vec<Node<Arc<Model>>>,
    models: Vec<ModelInstance>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Layer {
            name: name.to_string(),
            visible: true,
            nodes: Vec::new(),
            models: Vec::new(),
        }
    }

    pub fn add_node(&mut self, position: Vec3, rotation: Quat, scale: Vec3, model: Arc<Model>) {
        let exist = self.models.iter().find(|m| m.model.id == model.id);

        if exist.is_none() {
            let model_instance = ModelInstance {
                model: model.clone(),
                instance_buffer: None,
                instance_count: 0,
            };

            self.models.push(model_instance);
        };
        let node = Node::new(position, rotation, scale, model);
        self.nodes.push(node);
    }

    pub fn update(&mut self, gfx: &Gfx) {
        for model in &mut self.models {
            let instance_data = self
                .nodes
                .iter()
                .filter(|n| n.model().id == model.model.id)
                .map(|n| {
                    InstanceData::new(model.model.shader.instance_flags)
                        .model_transform(
                            n.transform().translation,
                            n.transform().rotation,
                            n.transform().scale,
                        )
                        .normal_rot(n.transform().rotation)
                        .build()
                })
                .collect::<Vec<_>>();

            match &model.instance_buffer {
                Some(instance_buffer) => {
                    gfx.update_instance_buffer(&instance_buffer, &instance_data);
                }
                None => {
                    model.instance_buffer = Some(gfx.create_instance_buffer(&instance_data));
                }
            }
            model.instance_count = instance_data.len();
        }
    }

    pub fn render<'a>(&'a self, mut batch: BatchBuilder<'a>) -> BatchBuilder<'a> {
        for instance in &self.models {
            batch = batch.draw_indexed(
                &instance.model,
                instance.instance_buffer.as_ref().unwrap(),
                instance.instance_count,
            );
        }

        batch
    }
}
