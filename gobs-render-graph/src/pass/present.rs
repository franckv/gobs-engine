use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{Command, Display, Image, ImageLayout};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::{SceneData, UniformLayout},
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct PresentPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
}

impl PresentPass {
    pub fn new(_ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Present,
            attachments: vec![String::from("draw")],
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

    fn push_layout(&self) -> Option<std::sync::Arc<UniformLayout>> {
        None
    }

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Present");

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

        Ok(())
    }
}
