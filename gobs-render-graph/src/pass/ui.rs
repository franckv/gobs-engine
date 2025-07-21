use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{GfxPipeline, ImageLayout, ImageUsage};
use gobs_resource::{
    entity::{camera::Camera, light::Light},
    geometry::VertexAttribute,
};

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

pub struct UiPass {
    ty: PassType,
    attachments: Vec<String>,
    color_clear: bool,
    material_pass: MaterialPass,
}

impl UiPass {
    pub fn new(
        ctx: &GfxContext,
        name: &str,
        color_clear: bool,
    ) -> Result<Arc<dyn RenderPass>, RenderError> {
        let scene_layout = SceneDataLayout::builder()
            .prop(SceneDataProp::CameraViewPort)
            .build();

        let object_layout = ObjectDataLayout::builder()
            .prop(ObjectDataProp::VertexBufferAddress)
            .build();

        let mut material_pass =
            MaterialPass::new(ctx, name, object_layout, scene_layout, true, true);

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

        Ok(Arc::new(Self {
            ty: PassType::Ui,
            attachments: vec![String::from("draw")],
            color_clear,
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

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn color_clear(&self) -> bool {
        self.color_clear
    }

    fn depth_clear(&self) -> bool {
        false
    }

    fn pipeline(&self) -> Option<Arc<GfxPipeline>> {
        None
    }

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        self.material_pass.push_layout()
    }

    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>> {
        self.material_pass.uniform_data_layout()
    }

    fn get_uniform_data(
        &self,
        _camera: &Camera,
        _camera_transform: &Transform,
        _light: &Light,
        _light_transform: &Transform,
    ) -> Vec<u8> {
        vec![]
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
