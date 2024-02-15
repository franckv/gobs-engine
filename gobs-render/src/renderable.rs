use std::{collections::HashSet, sync::Arc};

use gobs_core::Transform;

use crate::{
    context::Context,
    geometry::{Model, ModelId},
    material::MaterialInstance,
    pass::RenderPass,
    CommandBuffer,
};

use crate::resources::ModelResource;

#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    pub vertices: u32,
    pub indices: u32,
    pub models: u32,
    pub textures: u32,
    pub instances: u32,
    pub draws: u32,
    pub binds: u32,
    pub cpu_draw_time: f32,
    pub update_time: f32,
    models_set: HashSet<ModelId>,
}

impl RenderStats {
    pub fn reset(&mut self) {
        self.vertices = 0;
        self.indices = 0;
        self.models = 0;
        self.textures = 0;
        self.instances = 0;
        self.draws = 0;
        self.binds = 0;
        self.cpu_draw_time = 0.;
        self.update_time = 0.;
        self.models_set.clear();
    }

    pub fn add_model(&mut self, model: &Arc<Model>) {
        if !self.models_set.contains(&model.id) {
            self.models_set.insert(model.id);
            self.vertices += model
                .meshes
                .iter()
                .map(|(m, _)| m.vertices.len() as u32)
                .sum::<u32>();
            self.indices += model
                .meshes
                .iter()
                .map(|(m, _)| m.indices.len() as u32)
                .sum::<u32>();
            self.models += 1;
            self.textures += model
                .materials
                .values()
                .map(|m| m.textures.len() as u32)
                .sum::<u32>();
        }
        self.instances += 1;
    }
}

pub struct RenderObject {
    pub transform: Transform,
    pub pass: Arc<dyn RenderPass>,
    pub model: Arc<ModelResource>,
    pub material: Arc<MaterialInstance>,
    pub indices_offset: usize,
    pub indices_len: usize,
}

pub trait Renderable {
    fn resize(&mut self, width: u32, height: u32);
    fn draw(&self, ctx: &Context, pass: Arc<dyn RenderPass>, cmd: &CommandBuffer);
    fn stats(&self) -> RenderStats;
}
