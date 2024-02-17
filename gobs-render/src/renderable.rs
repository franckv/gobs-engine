use std::collections::HashMap;
use std::sync::Arc;

use gobs_core::Transform;

use crate::pass::PassId;
use crate::resources::ModelResource;
use crate::stats::RenderStats;
use crate::{context::Context, material::MaterialInstance, pass::RenderPass};

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
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

    pub fn add_object(&mut self, object: RenderObject) {
        self.render_stats.add_object(&object);
        self.render_list.push(object);
    }

    pub fn add_scene_data(&mut self, scene_data: Vec<u8>, pass_id: PassId) {
        self.scene_data.insert(pass_id, scene_data);
    }

    pub fn scene_data(&self, pass_id: PassId) -> Option<&Vec<u8>> {
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
