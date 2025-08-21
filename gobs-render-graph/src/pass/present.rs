use std::sync::Arc;

use gobs_core::{ImageExtent2D, logger};
use gobs_gfx::{Command, Display, Image, ImageLayout};
use gobs_render_low::{FrameData, GfxContext, RenderError, RenderObject, RenderStats, SceneData};
use gobs_resource::geometry::VertexAttribute;

use crate::{
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

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
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

        let mut stats = RenderStats::default();

        let cmd = &frame.command;

        if let Some(render_target) = ctx.display.get_render_target() {
            cmd.transition_image_layout(
                &mut resource_manager.image_write("draw"),
                ImageLayout::TransferSrc,
            );

            cmd.transition_image_layout(render_target, ImageLayout::TransferDst);

            cmd.copy_image_to_image(
                &resource_manager.image_read("draw"),
                draw_extent,
                render_target,
                render_target.extent(),
            );
        }

        stats.finish(self.id);

        Ok(())
    }
}
