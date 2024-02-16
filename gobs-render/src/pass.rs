use std::sync::Arc;

use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{
    image::{Image, ImageExtent2D},
    pipeline::Pipeline,
};

use crate::{
    context::Context, geometry::VertexFlag, graph::RenderError, renderable::RenderBatch,
    CommandBuffer,
};

pub mod compute;
pub mod forward;
pub mod ui;
pub mod wire;

#[derive(Clone, Copy, Debug)]
pub enum PassType {
    Compute,
    Forward,
    Wire,
    Ui,
}

pub type PassId = Uuid;

pub trait RenderPass {
    fn id(&self) -> PassId;
    fn name(&self) -> &str;
    fn ty(&self) -> PassType;
    fn pipeline(&self) -> Option<Arc<Pipeline>>;
    fn vertex_flags(&self) -> Option<VertexFlag>;
    fn push_layout(&self) -> Option<Arc<UniformLayout>>;
    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>>;
    fn render(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
        //draw_cmd: &mut dyn FnMut(Arc<dyn RenderPass>, &CommandBuffer, &mut Vec<RenderObject>),
    ) -> Result<(), RenderError>;
}
