use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{GfxCommand, GfxPipeline};
use gobs_resource::{entity::uniform::UniformLayout, geometry::VertexFlag};

use crate::{
    RenderError,
    batch::RenderBatch,
    context::Context,
    graph::ResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct DummyPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
}

impl DummyPass {
    pub fn new(_ctx: &Context, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Dummy,
            attachments: vec![],
        }))
    }
}

impl RenderPass for DummyPass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn pipeline(&self) -> Option<Arc<GfxPipeline>> {
        None
    }

    fn vertex_flags(&self) -> Option<VertexFlag> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        None
    }

    fn uniform_data_layout(&self) -> Option<std::sync::Arc<UniformLayout>> {
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
        _ctx: &mut Context,
        _cmd: &GfxCommand,
        _resource_manager: &ResourceManager,
        _batch: &mut RenderBatch,
        _draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!("Rendering {}", &self.name);
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
