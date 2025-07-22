use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{
    BindingGroupType, CompareOp, CullMode, DescriptorStage, DescriptorType, DynamicStateElem,
    FrontFace, GfxPipeline, GraphicsPipelineBuilder, ImageLayout, ImageUsage, Pipeline,
    PolygonMode, Rect2D, Viewport,
};
use gobs_render_low::{
    GfxContext, ObjectDataLayout, ObjectDataProp, RenderError, RenderObject, SceneData,
    SceneDataLayout, SceneDataProp, UniformLayout,
};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData,
    graph::GraphResourceManager,
    pass::{
        AttachmentAccess, AttachmentType, PassId, PassType, RenderPass, material::MaterialPass,
    },
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

pub struct DepthPass {
    ty: PassType,
    attachments: Vec<String>,
    material_pass: MaterialPass,
}

impl DepthPass {
    pub fn new(ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let scene_layout = SceneDataLayout::builder()
            .prop(SceneDataProp::CameraViewProj)
            .build();

        let object_layout = ObjectDataLayout::builder()
            .prop(ObjectDataProp::WorldMatrix)
            .prop(ObjectDataProp::VertexBufferAddress)
            .build();

        let push_layout = object_layout.uniform_layout();

        let mut material_pass =
            MaterialPass::new(ctx, name, object_layout, scene_layout, false, true);

        let extent = ctx.extent();
        let extent = ImageExtent2D::new(
            extent.width.max(FRAME_WIDTH),
            extent.height.max(FRAME_HEIGHT),
        );

        material_pass
            .add_attachment("depth", AttachmentType::Depth, AttachmentAccess::ReadWrite)
            .with_usage(ImageUsage::Depth)
            .with_extent(extent)
            .with_format(ctx.depth_format)
            .with_clear(true)
            .with_layout(ImageLayout::Depth);

        let vertex_attributes = VertexAttribute::POSITION;

        let pipeline = GfxPipeline::graphics(name, &ctx.device)
            .vertex_shader("depth.spv", "main")?
            .pool_size(ctx.frames_in_flight)
            .push_constants(push_layout.size())
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::Vertex)
            .polygon_mode(PolygonMode::Fill)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(None, Some(ctx.depth_format))
            .depth_test_enable(true, CompareOp::Less)
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        material_pass.set_fixed_pipeline(pipeline.clone(), vertex_attributes);

        Ok(Arc::new(Self {
            ty: PassType::Depth,
            attachments: vec![String::from("depth")],
            material_pass,
        }))
    }
}

impl RenderPass for DepthPass {
    fn id(&self) -> PassId {
        self.material_pass.id()
    }

    fn name(&self) -> &str {
        self.material_pass.name()
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn attachments(&self) -> &[String] {
        &self.attachments
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
