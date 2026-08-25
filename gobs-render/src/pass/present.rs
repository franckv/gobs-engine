use gobs_core::logger;
use gobs_render_graph::{FrameData, GraphResourceManager};
use gobs_render_hal::ImageLayout;

use crate::{GfxContext, RenderError};

pub struct PresentPassData {
    render_target: String,
}

impl PresentPassData {
    pub fn new(render_target: &str) -> Self {
        Self {
            render_target: render_target.to_string(),
        }
    }
}

pub struct PresentPass;

impl PresentPass {
    pub fn render(
        ctx: &mut GfxContext,
        pass_data: &PresentPassData,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Present");

        let cmd = &mut frame.command;

        if let Some(render_target) = ctx.hal().get_render_target() {
            cmd.transition_image_layout(
                ctx.hal_mut(),
                resource_manager.image(&pass_data.render_target),
                ImageLayout::TransferSrc,
            );

            cmd.transition_image_layout(ctx.hal_mut(), render_target, ImageLayout::TransferDst);

            cmd.copy_image_to_image(
                ctx.hal(),
                resource_manager.image(&pass_data.render_target),
                render_target,
            );
        }

        Ok(())
    }
}
