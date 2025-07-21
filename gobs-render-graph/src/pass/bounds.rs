use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{
    BindingGroupType, CullMode, DescriptorStage, DescriptorType, DynamicStateElem, FrontFace,
    GfxPipeline, GraphicsPipelineBuilder, ImageLayout, ImageUsage, Pipeline, PolygonMode, Rect2D,
    Viewport,
};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::{
        ObjectDataLayout, ObjectDataProp, SceneData, SceneDataLayout, SceneDataProp, UniformLayout,
    },
    graph::GraphResourceManager,
    pass::{
        AttachmentAccess, AttachmentType, PassId, PassType, RenderPass, material::MaterialPass,
    },
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

pub struct BoundsPass {
    ty: PassType,
    attachments: Vec<String>,
    material_pass: MaterialPass,
}

impl BoundsPass {
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
            .add_attachment("draw", AttachmentType::Color, AttachmentAccess::ReadWrite)
            .with_usage(ImageUsage::Color)
            .with_extent(extent)
            .with_format(ctx.color_format)
            .with_clear(false)
            .with_layout(ImageLayout::Color);

        let vertex_attributes = VertexAttribute::POSITION;

        let pipeline = GfxPipeline::graphics(name, &ctx.device)
            .vertex_shader("wire.spv", "vertex_main")?
            .fragment_shader("wire.spv", "fragment_main")?
            .pool_size(ctx.frames_in_flight)
            .push_constants(push_layout.size())
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
            ty: PassType::Bounds,
            attachments: vec![String::from("draw")],
            material_pass,
        }))
    }
}

impl RenderPass for BoundsPass {
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
        tracing::debug!(target: "render", "Draw {}", &self.name());

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
