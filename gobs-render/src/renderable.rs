use std::collections::HashMap;
use std::{collections::HashSet, sync::Arc};

use gobs_core::entity::uniform::UniformData;
use gobs_core::Transform;

use crate::pass::PassId;
use crate::{context::Context, geometry::ModelId, material::MaterialInstance, pass::RenderPass};

use crate::resources::ModelResource;

#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    pub ui_vertices: u32,
    pub scene_vertices: u32,
    pub ui_indices: u32,
    pub scene_indices: u32,
    pub ui_models: u32,
    pub scene_models: u32,
    pub ui_textures: u32,
    pub scene_textures: u32,
    pub ui_instances: u32,
    pub scene_instances: u32,
    pub draws: u32,
    pub binds: u32,
    pub cpu_draw_time: f32,
    pub gpu_draw_time: f32,
    pub update_time: f32,
    models_set: HashSet<ModelId>,
}

impl RenderStats {
    pub fn reset(&mut self) {
        self.ui_vertices = 0;
        self.scene_vertices = 0;
        self.ui_indices = 0;
        self.scene_indices = 0;
        self.ui_models = 0;
        self.scene_models = 0;
        self.ui_textures = 0;
        self.scene_textures = 0;
        self.ui_instances = 0;
        self.scene_instances = 0;
        self.draws = 0;
        self.binds = 0;
        self.models_set.clear();
    }

    pub fn add_object(&mut self, object: &RenderObject, ui: bool) {
        if !self.models_set.contains(&object.model.model.id) {
            self.models_set.insert(object.model.model.id);
            let vertices = object
                .model
                .model
                .meshes
                .iter()
                .map(|(m, _)| m.vertices.len() as u32)
                .sum::<u32>();
            let indices = object
                .model
                .model
                .meshes
                .iter()
                .map(|(m, _)| m.indices.len() as u32)
                .sum::<u32>();
            let models = 1;
            let textures = object
                .model
                .model
                .materials
                .values()
                .map(|m| m.textures.len() as u32)
                .sum::<u32>();
            if ui {
                self.ui_vertices += vertices;
                self.ui_indices += indices;
                self.ui_models += models;
                self.ui_textures += textures;
            } else {
                self.scene_vertices += vertices;
                self.scene_indices += indices;
                self.scene_models += models;
                self.scene_textures += textures;
            }
        }
        if ui {
            self.ui_instances += 1;
        } else {
            self.scene_instances += 1;
        }
    }
}

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, UniformData>,
    pub(crate) render_stats: RenderStats,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            render_stats: RenderStats::default(),
        }
    }

    pub fn reset(&mut self) {
        self.render_list.clear();
        self.scene_data.clear();
        self.render_stats.reset();
    }

    pub fn add_object(&mut self, object: RenderObject, ui: bool) {
        self.render_stats.add_object(&object, ui);
        self.render_list.push(object);
    }

    pub fn add_scene_data(&mut self, scene_data: UniformData, pass_id: PassId) {
        self.scene_data.insert(pass_id, scene_data);
    }

    pub fn scene_data(&self, pass_id: PassId) -> Option<&UniformData> {
        self.scene_data.get(&pass_id)
    }

    pub fn stats_mut(&mut self) -> &mut RenderStats {
        &mut self.render_stats
    }

    pub fn finish(&mut self) {
        self.render_list.sort_by(|a, b| {
            // sort order: pass, material: model
            if a.pass.id() == b.pass.id() {
                if a.material.id == b.material.id {
                    a.model.model.id.cmp(&b.model.model.id)
                } else {
                    a.material.id.cmp(&b.material.id)
                }
            } else {
                a.pass.id().cmp(&b.pass.id())
            }
        });
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
    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch);
}
