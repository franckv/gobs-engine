use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_render_low::{GfxContext, RenderError, RenderObject, SceneData, UniformLayout};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData, GraphConfig,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass, material::MaterialPass},
};

pub struct UiPass {
    ty: PassType,
    material_pass: MaterialPass,
}

impl UiPass {
    pub fn new(ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let material_pass =
            GraphConfig::load_pass(ctx, "graph.ron", name).ok_or(RenderError::PassNotFound)?;

        Ok(Arc::new(Self {
            ty: PassType::Ui,
            material_pass,
        }))
    }
}

impl RenderPass for UiPass {
    fn id(&self) -> PassId {
        self.material_pass.id()
    }

    fn name(&self) -> &str {
        self.material_pass.name()
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        self.material_pass.push_layout()
    }

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        render_list: &[RenderObject],
        scene_data: &SceneData,
        _draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Draw {}", &self.material_pass.name());

        let cmd = &frame.command;

        self.material_pass
            .transition_attachments(cmd, resource_manager);

        self.material_pass.begin_pass(cmd, resource_manager);

        self.material_pass
            .render(ctx, frame, cmd, render_list, scene_data);

        self.material_pass.end_pass(cmd);

        Ok(())
    }
}
