use gobs_core::logger;
use gobs_render_hal::ImageLayout;

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::SceneData,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct PresentPass {
    id: PassId,
    name: String,
    ty: PassType,
    render_target: String,
}

impl PresentPass {
    pub fn new(_ctx: &GfxContext, name: &str, render_target: &str) -> Self {
        Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Present,
            render_target: render_target.to_string(),
        }
    }
}

impl RenderPass for PresentPass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Present");

        let cmd = &frame.command;

        if let Some(render_target) = ctx.hal.get_render_target() {
            cmd.transition_image_layout(
                ctx.hal.as_mut(),
                resource_manager.image(&self.render_target),
                ImageLayout::TransferSrc,
            );

            cmd.transition_image_layout(ctx.hal.as_mut(), render_target, ImageLayout::TransferDst);

            cmd.copy_image_to_image(
                ctx.hal.as_ref(),
                resource_manager.image(&self.render_target),
                render_target,
            );
        }

        Ok(())
    }
}
