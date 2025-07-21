use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{
    BindingGroupType, CullMode, DescriptorStage, DescriptorType, DynamicStateElem, FrontFace,
    GfxPipeline, GraphicsPipelineBuilder, ImageLayout, ImageUsage, Pipeline, PolygonMode, Rect2D,
    Viewport,
};
use gobs_resource::{
    entity::{camera::Camera, light::Light},
    geometry::VertexAttribute,
};

use crate::pass::AttachmentAccess;
use crate::pass::AttachmentType;
use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::{
        ObjectDataLayout, ObjectDataProp, SceneData, SceneDataLayout, SceneDataProp, UniformLayout,
        UniformPropData,
    },
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass, material::MaterialPass},
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

pub struct WirePass {
    ty: PassType,
    attachments: Vec<String>,
    pipeline: Arc<GfxPipeline>,
    material_pass: MaterialPass,
}

impl WirePass {
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
            ty: PassType::Wire,
            attachments: vec![String::from("draw")],
            pipeline,
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

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn color_clear(&self) -> bool {
        false
    }

    fn depth_clear(&self) -> bool {
        false
    }

    fn pipeline(&self) -> Option<Arc<GfxPipeline>> {
        Some(self.pipeline.clone())
    }

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        self.material_pass.vertex_attributes
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        self.material_pass.push_layout()
    }

    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>> {
        self.material_pass.uniform_data_layout()
    }

    fn get_uniform_data(
        &self,
        camera: &Camera,
        camera_transform: &Transform,
        _light: &Light,
        _light_transform: &Transform,
    ) -> Vec<u8> {
        self.material_pass
            .uniform_data_layout()
            .unwrap()
            .data(&[UniformPropData::Mat4F(
                camera
                    .view_proj(camera_transform.translation())
                    .to_cols_array_2d(),
            )])
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
