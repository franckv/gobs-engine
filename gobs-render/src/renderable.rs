use std::sync::Arc;

use gobs_core::Transform;

use crate::{
    batch::RenderBatch, context::Context, material::MaterialInstance, pass::RenderPass,
    resources::ModelResource,
};

pub struct RenderObject {
    pub transform: Transform,
    pub pass: Arc<dyn RenderPass>,
    pub model: Arc<ModelResource>,
    pub material: Option<Arc<MaterialInstance>>,
    pub vertices_offset: u64,
    pub indices_offset: usize,
    pub indices_len: usize,
}

pub trait Renderable {
    fn resize(&mut self, width: u32, height: u32);
    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch);
}
