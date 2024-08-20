use std::sync::Arc;

use gobs_core::Transform;

use crate::{batch::RenderBatch, context::Context, pass::RenderPass, resources::GPUMesh};

pub struct RenderObject {
    pub transform: Transform,
    pub pass: Arc<dyn RenderPass>,
    pub mesh: GPUMesh,
}

pub trait Renderable {
    fn resize(&mut self, width: u32, height: u32);
    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch);
}
