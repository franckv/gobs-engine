use std::sync::Arc;

use uuid::Uuid;

use gobs_core::entity::uniform::{UniformData, UniformLayout};
use gobs_vulkan::{
    image::{Image, ImageExtent2D},
    pipeline::Pipeline,
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::RenderError,
    renderable::{RenderObject, RenderStats},
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
        self: Arc<Self>,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &mut dyn FnMut(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError>;
    fn draw(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_list: &[RenderObject],
        scene_data: Option<UniformData>,
        render_stats: &mut RenderStats,
    );
}
