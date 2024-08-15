use std::sync::Arc;

use gobs_core::Transform;
use gobs_gfx::{BindingGroupType, Command, DescriptorType, ImageExtent2D, ImageLayout, Pipeline};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformLayout},
    geometry::VertexFlag,
};

use crate::{
    batch::RenderBatch,
    context::Context,
    graph::{RenderError, ResourceManager},
    pass::{PassId, PassType, RenderPass},
    GfxBindingGroup, GfxCommand, GfxPipeline,
};

pub(crate) struct FrameData {
    pub draw_bindings: GfxBindingGroup,
}

impl FrameData {
    pub fn new(draw_bindings: GfxBindingGroup) -> Self {
        FrameData { draw_bindings }
    }
}

pub struct ComputePass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    frame_data: Vec<FrameData>,
    pub pipeline: Arc<GfxPipeline>,
}

impl ComputePass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let pipeline_builder = GfxPipeline::compute(name, &ctx.device);

        let pipeline = pipeline_builder
            .shader("sky.comp.spv", "main")
            .binding_group(BindingGroupType::ComputeData)
            .binding(DescriptorType::StorageImage)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| {
                FrameData::new(
                    pipeline
                        .create_binding_group(BindingGroupType::ComputeData)
                        .unwrap(),
                )
            })
            .collect();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Compute,
            attachments: vec![String::from("draw")],
            frame_data,
            pipeline,
        })
    }
}

impl RenderPass for ComputePass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
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

    fn vertex_flags(&self) -> Option<VertexFlag> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        None
    }

    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>> {
        None
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
        ctx: &Context,
        cmd: &GfxCommand,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        log::debug!("Draw compute");
        cmd.begin_label("Draw compute");

        let draw_attach = &self.attachments[0];

        let frame_id = ctx.frame_id();
        let draw_bindings = &self.frame_data[frame_id].draw_bindings;

        draw_bindings
            .update()
            .bind_image(
                &resource_manager.image_read(draw_attach),
                ImageLayout::General,
            )
            .end();

        batch.stats_mut().binds += 1;

        cmd.transition_image_layout(
            &mut resource_manager.image_write(draw_attach),
            ImageLayout::General,
        );

        cmd.bind_pipeline(&self.pipeline);
        cmd.bind_resource(&draw_bindings);
        batch.stats_mut().binds += 2;

        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);
        batch.stats_mut().draws += 1;

        cmd.end_label();

        Ok(())
    }
}
