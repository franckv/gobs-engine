use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{Command, Display, GfxCommand, GfxPipeline, Image, ImageLayout};

use crate::{
    RenderError,
    batch::RenderBatch,
    context::Context,
    graph::ResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct PresentPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
}

impl PresentPass {
    pub fn new(_ctx: &Context, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
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

    fn pipeline(&self) -> Option<std::sync::Arc<GfxPipeline>> {
        None
    }

    fn vertex_attributes(&self) -> Option<gobs_resource::geometry::VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<std::sync::Arc<gobs_resource::entity::uniform::UniformLayout>> {
        None
    }

    fn uniform_data_layout(
        &self,
    ) -> Option<std::sync::Arc<gobs_resource::entity::uniform::UniformLayout>> {
        None
    }

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn color_clear(&self) -> bool {
        false
    }

    fn depth_clear(&self) -> bool {
        false
    }

    fn render(
        &self,
        ctx: &mut Context,
        cmd: &GfxCommand,
        resource_manager: &ResourceManager,
        _batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!("Present");

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

    fn get_uniform_data(
        &self,
        _camera: &gobs_resource::entity::camera::Camera,
        _camera_transform: &gobs_core::Transform,
        _light: &gobs_resource::entity::light::Light,
        _light_transform: &gobs_core::Transform,
    ) -> Vec<u8> {
        vec![]
    }
}
