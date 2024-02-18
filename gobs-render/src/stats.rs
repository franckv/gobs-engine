use std::collections::{HashMap, HashSet};

use crate::{geometry::ModelId, pass::PassId, renderable::RenderObject};

#[derive(Clone, Debug, Default)]
pub struct PassStats {
    pub vertices: u32,
    pub indices: u32,
    pub models: u32,
    pub textures: u32,
    pub instances: u32,
}

#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    pub draws: u32,
    pub binds: u32,
    pub cpu_draw_time: f32,
    pub cpu_draw_pre: f32,
    pub cpu_draw_mid: f32,
    pub cpu_draw_post: f32,
    pub gpu_draw_time: f32,
    pub update_time: f32,
    pub pass_stats: HashMap<PassId, PassStats>,
    models_set: HashSet<ModelId>,
}

impl RenderStats {
    pub fn reset(&mut self) {
        self.draws = 0;
        self.binds = 0;
        self.cpu_draw_pre = 0.;
        self.cpu_draw_mid = 0.;
        self.cpu_draw_post = 0.;
        self.pass_stats.clear();
        self.models_set.clear();
    }

    pub fn add_object(&mut self, object: &RenderObject) {
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

            let pass_stat = self
                .pass_stats
                .entry(object.pass.id())
                .or_insert(PassStats::default());

            pass_stat.vertices += vertices;
            pass_stat.indices += indices;
            pass_stat.models += models;
            pass_stat.textures += textures;
            pass_stat.instances += 1;
        }
    }
}
