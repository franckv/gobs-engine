use std::sync::Arc;

use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{ImageLayout, SceneData};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct PresentPass {
    id: PassId,
    name: String,
    ty: PassType,
}

impl PresentPass {
    pub fn new(_ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Present,
        }))
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
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Present");

        let cmd = &frame.command;

        let render_target = ctx.hal.get_render_target();
        let render_extent = ctx.hal.get_extent();

        cmd.transition_image_layout(
            ctx.hal.as_mut(),
            resource_manager.image("draw"),
            ImageLayout::TransferSrc,
        );

        cmd.transition_image_layout(ctx.hal.as_mut(), render_target, ImageLayout::TransferDst);

        cmd.copy_image_to_image(
            ctx.hal.as_ref(),
            resource_manager.image("draw"),
            draw_extent,
            render_target,
            render_extent,
        );

        Ok(())
    }
}
