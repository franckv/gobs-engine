use std::sync::Arc;

use gobs_core::ImageExtent2D;
use gobs_gfx::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, Command, ComputePipelineBuilder,
    DescriptorType, GfxBindingGroup, GfxPipeline, ImageLayout, Pipeline,
};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::{SceneData, UniformLayout},
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub(crate) struct PassFrameData {
    pub draw_bindings: GfxBindingGroup,
}

impl PassFrameData {
    pub fn new(draw_bindings: GfxBindingGroup) -> Self {
        PassFrameData { draw_bindings }
    }
}

pub struct ComputePass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    frame_data: Vec<PassFrameData>,
    pub pipeline: Arc<GfxPipeline>,
}

impl ComputePass {
    pub fn new(ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let pipeline_builder = GfxPipeline::compute(name, &ctx.device);

        let pipeline = pipeline_builder
            .shader("sky.comp.spv", "main")?
            .binding_group(BindingGroupType::ComputeData)
            .binding(DescriptorType::StorageImage)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| {
                PassFrameData::new(
                    pipeline
                        .create_binding_group(BindingGroupType::ComputeData)
                        .unwrap(),
                )
            })
            .collect();

        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Compute,
            attachments: vec![String::from("draw")],
            frame_data,
            pipeline,
        }))
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

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        None
    }

    fn render(
        &self,
        _ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Draw compute");

        let cmd = &frame.command;

        cmd.begin_label("Draw compute");

        let draw_attach = &self.attachments[0];

        let pass_frame = &self.frame_data[frame.id];

        let draw_bindings = &pass_frame.draw_bindings;

        draw_bindings
            .update()
            .bind_image(
                &resource_manager.image_read(draw_attach),
                ImageLayout::General,
            )
            .end();

        cmd.transition_image_layout(
            &mut resource_manager.image_write(draw_attach),
            ImageLayout::General,
        );

        cmd.bind_pipeline(&self.pipeline);
        cmd.bind_resource(draw_bindings);

        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);

        cmd.end_label();

        Ok(())
    }
}
