use gobs_core::logger;
use gobs_render_graph::{
    AttachmentAccess, AttachmentType, FrameData, FrameGraph, GraphResourceManager,
};
use gobs_render_hal::{Barrier, BarrierAccess, BarrierStage, BarrierSyncScope, ImageLayout};

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

        let image = resource_manager.image(&pass_data.render_target);
        let layout = ctx.get_image_layout(image);

        if let Some(render_target) = ctx.get_render_target() {
            let barrier = Barrier::new("transfer src")
                .image(&pass_data.render_target)
                .layouts(layout, ImageLayout::TransferSrc)
                .transition(
                    FrameGraph::barrier_scope(AttachmentType::Color, AttachmentAccess::Write),
                    FrameGraph::barrier_scope(AttachmentType::Transfer, AttachmentAccess::Read),
                );
            cmd.set_image_barrier(ctx, &barrier, image);

            let barrier = Barrier::new("transfer dst")
                .image("swapchain")
                .layouts(ImageLayout::Undefined, ImageLayout::TransferDst)
                .transition(
                    BarrierSyncScope {
                        stage: BarrierStage::TopOfPipe,
                        access: BarrierAccess::empty(),
                    },
                    FrameGraph::barrier_scope(AttachmentType::Transfer, AttachmentAccess::Write),
                );
            cmd.set_image_barrier(ctx, &barrier, render_target);

            cmd.copy_image_to_image(ctx, image, render_target);
        }

        Ok(())
    }
}
