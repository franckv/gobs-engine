use gobs_core::logger;
use gobs_render_hal::ImageLayout;

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::SceneData,
    graph::GraphResourceManager,
    pass::{PassId, RenderPass, metadata::PassMetaData},
};

pub struct PresentPass {
    pub metadata: PassMetaData,
    render_target: String,
}

impl PresentPass {
    pub fn new(_ctx: &GfxContext, metadata: PassMetaData, render_target: &str) -> Self {
        Self {
            metadata,
            render_target: render_target.to_string(),
        }
    }
}

impl RenderPass for PresentPass {
    fn id(&self) -> PassId {
        self.metadata.id
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PassMetaData {
        &self.metadata
    }

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Present");

        let cmd = &mut frame.command;

        if let Some(render_target) = ctx.hal().get_render_target() {
            cmd.transition_image_layout(
                ctx.hal_mut(),
                resource_manager.image(&self.render_target),
                ImageLayout::TransferSrc,
            );

            cmd.transition_image_layout(ctx.hal_mut(), render_target, ImageLayout::TransferDst);

            cmd.copy_image_to_image(
                ctx.hal(),
                resource_manager.image(&self.render_target),
                render_target,
            );
        }

        Ok(())
    }
}
