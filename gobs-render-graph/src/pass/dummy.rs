use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_render_low::{GfxContext, RenderError, RenderObject, SceneData, UniformLayout};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct DummyPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
}

impl DummyPass {
    pub fn new(_ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
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

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        None
    }

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn render(
        &self,
        _ctx: &mut GfxContext,
        _frame: &FrameData,
        _resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
        _draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Rendering {}", &self.name);
        Ok(())
    }
}
