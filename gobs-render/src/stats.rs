use std::collections::{HashMap, HashSet};

use gobs_gfx::{Buffer, BufferId};

use crate::{ModelId, pass::PassId, renderable::RenderObject};

#[derive(Clone, Debug, Default)]
pub struct PassStats {
    pub vertices: u32,
    pub indices: u32,
    pub models: u32,
    pub textures: u32,
    pub instances: u32,
    pub draws: u32,
    pub binds: u32,
    pub draw_time: f32,
    pub update_time: f32,
}

#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    draws: u32,
    binds: u32,
    cpu_draw_time: f32,
    pub gpu_draw_time: f32,
    update_time: f32,
    pub fps: u32,
    pub pass_stats: HashMap<PassId, PassStats>,
    models_set: HashSet<(PassId, ModelId)>,
    indices_set: HashSet<(PassId, BufferId, usize)>,
}

impl RenderStats {
    pub fn reset(&mut self) {
        self.draws = 0;
        self.binds = 0;
        self.pass_stats.clear();
        self.models_set.clear();
        self.indices_set.clear();
    }

    pub fn cpu_draw_time_reset(&mut self, update: bool) {
        if update {
            self.cpu_draw_time = 0.;
        };
    }

    pub fn cpu_draw_time_add(&mut self, t: f32, pass_id: PassId, update: bool) {
        if update {
            self.cpu_draw_time += t
        };

        let pass_stats = self.pass_stats.entry(pass_id).or_default();

        pass_stats.draw_time = t;
    }

    pub fn cpu_draw_time(&self) -> f32 {
        self.cpu_draw_time
    }

    pub fn update_time_reset(&mut self, update: bool) {
        if update {
            self.update_time = 0.;
        };
    }

    pub fn update_time_add(&mut self, t: f32, pass_id: PassId, update: bool) {
        if update {
            self.update_time += t
        };

        let pass_stats = self.pass_stats.entry(pass_id).or_default();

        pass_stats.update_time = t;
    }

    pub fn update_time(&self) -> f32 {
        self.update_time
    }

    pub fn binds(&self) -> u32 {
        self.binds
    }

    pub fn bind(&mut self, pass_id: PassId) {
        self.binds += 1;

        let pass_stats = self.pass_stats.entry(pass_id).or_default();

        pass_stats.binds += 1
    }

    pub fn draws(&self) -> u32 {
        self.draws
    }

    pub fn draw(&mut self, pass_id: PassId) {
        self.draws += 1;

        let pass_stats = self.pass_stats.entry(pass_id).or_default();

        pass_stats.draws += 1
    }

    pub fn add_object(&mut self, object: &RenderObject) {
        let key = (
            object.pass.id(),
            object.mesh.index_buffer.id(),
            object.mesh.indices_offset,
        );

        if !self.indices_set.contains(&key) {
            self.indices_set.insert(key);

            let indices = object.mesh.indices_len as u32;
            let vertices = object.mesh.vertices_count as u32;
            let models = 1;

            let pass_stat = self.pass_stats.entry(object.pass.id()).or_default();

            pass_stat.indices += indices;
            pass_stat.vertices += vertices;
            pass_stat.models += models;
            pass_stat.instances += 1;
        }
        // if !self
        //     .models_set
        //     .contains(&(object.pass.id(), object.model.id))
        // {
        //     self.models_set.insert((object.pass.id(), object.model.id));
        //     let vertices = object
        //         .model
        //         .meshes
        //         .iter()
        //         .map(|(m, _)| m.vertices.len() as u32)
        //         .sum::<u32>();
        //     let indices = object
        //         .model
        //         .meshes
        //         .iter()
        //         .map(|(m, _)| m.indices.len() as u32)
        //         .sum::<u32>();
        //     let models = 1;
        //     let textures = object
        //         .model
        //         .materials
        //         .values()
        //         .map(|m| m.textures.len() as u32)
        //         .sum::<u32>();
        //
        //     let pass_stat = self.pass_stats.entry(object.pass.id()).or_default();
        //
        //     pass_stat.vertices += vertices;
        //     pass_stat.indices += indices;
        //     pass_stat.models += models;
        //     pass_stat.textures += textures;
        //     pass_stat.instances += 1;
        // }
    }
}
