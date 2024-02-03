use gobs_vulkan::image::{Image, ImageExtent2D};

use crate::{context::Context, graph::RenderError, CommandBuffer};

pub mod compute;
pub mod forward;

#[derive(Clone, Copy, Debug)]
pub enum PassType {
    Compute,
    Forward,
}

pub trait RenderPass {
    fn name(&self) -> &str;
    fn ty(&self) -> PassType;
    fn render<F>(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &F,
    ) -> Result<(), RenderError>
    where
        F: Fn(PassType, &str, &CommandBuffer);
}
