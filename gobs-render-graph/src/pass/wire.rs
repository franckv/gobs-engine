use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{
    BindingGroupType, CullMode, DescriptorStage, DescriptorType, DynamicStateElem, FrontFace,
    GfxPipeline, GraphicsPipelineBuilder, Pipeline, PolygonMode, Rect2D, Viewport,
};
use gobs_render_low::{GfxContext, RenderError, RenderObject, SceneData, UniformLayout};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData, GraphConfig,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass, material::MaterialPass},
};

pub struct WirePass {
    ty: PassType,
    material_pass: MaterialPass,
}

impl WirePass {
    pub fn new(ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let mut material_pass =
            GraphConfig::load_pass(ctx, "graph.ron", name).ok_or(RenderError::PassNotFound)?;

        let vertex_attributes = VertexAttribute::POSITION;

        let pipeline = GfxPipeline::graphics(name, &ctx.device)
            .vertex_shader("wire.spv", "vertex_main")?
            .fragment_shader("wire.spv", "fragment_main")?
            .pool_size(ctx.frames_in_flight)
            .push_constants(material_pass.push_constant_size())
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .polygon_mode(PolygonMode::Line)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(Some(ctx.color_format), Some(ctx.depth_format))
            .depth_test_disable()
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        material_pass.set_fixed_pipeline(pipeline.clone(), vertex_attributes);

        Ok(Arc::new(Self {
            ty: PassType::Wire,
            material_pass,
        }))
    }
}

impl RenderPass for WirePass {
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
        self.material_pass.vertex_attributes
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
