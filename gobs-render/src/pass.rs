use std::sync::Arc;

use gobs_vulkan::{
    image::{Image, ImageExtent2D},
    pipeline::Pipeline,
};
use uuid::Uuid;

use crate::{context::Context, geometry::VertexFlag, graph::RenderError, CommandBuffer};

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
    fn render(
        self: Arc<Self>,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &dyn Fn(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError>;
}
