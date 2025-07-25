use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::GfxPipeline;
use gobs_render_low::{GfxContext, RenderError, RenderObject, SceneData, UniformLayout};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData, GraphConfig,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass, material::MaterialPass},
};

pub struct SelectPass {
    ty: PassType,
    material_pass: MaterialPass,
}

impl SelectPass {
    pub fn new(
        ctx: &GfxContext,
        name: &str,
        pipeline: Arc<GfxPipeline>,
    ) -> Result<Arc<dyn RenderPass>, RenderError> {
        let mut material_pass =
            GraphConfig::load_pass(ctx, "graph.ron", name).ok_or(RenderError::PassNotFound)?;

        material_pass.set_fixed_pipeline(pipeline.clone());

        Ok(Arc::new(Self {
            ty: PassType::Select,
            material_pass,
        }))
    }
}

impl RenderPass for SelectPass {
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
        self.material_pass.vertex_attributes()
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
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        self.material_pass.render(
            ctx,
            frame,
            resource_manager,
            render_list,
            scene_data,
            draw_extent,
        )
    }
}
