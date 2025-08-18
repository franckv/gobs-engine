use std::sync::Arc;

use gobs_core::{ImageExtent2D, logger};
use gobs_gfx::{
    BindingGroup, BindingGroupLayout, BindingGroupPool, BindingGroupType, BindingGroupUpdates,
    Command, ComputePipelineBuilder, DescriptorType, GfxBindingGroup, GfxBindingGroupLayout,
    GfxBindingGroupPool, GfxPipeline, ImageLayout, Pipeline,
};
use gobs_render_low::{
    FrameData, GfxContext, RenderError, RenderObject, RenderStats, SceneData, UniformLayout,
};
use gobs_resource::geometry::VertexAttribute;

use crate::{
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
    _ds_pool: GfxBindingGroupPool,
}

impl ComputePass {
    pub fn new(ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let pipeline_builder = GfxPipeline::compute(name, ctx.device.clone());

        let binding_layout = GfxBindingGroupLayout::new(BindingGroupType::ComputeData).add_binding(
            DescriptorType::StorageImage,
            gobs_gfx::DescriptorStage::Compute,
        );

        let mut _ds_pool = GfxBindingGroupPool::new(
            ctx.device.clone(),
            ctx.frames_in_flight,
            binding_layout.clone(),
        );

        let pipeline = pipeline_builder
            .shader("sky.comp.spv", "main")?
            .binding_group(binding_layout)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| PassFrameData::new(_ds_pool.allocate()))
            .collect();

        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Compute,
            attachments: vec![String::from("draw")],
            frame_data,
            pipeline,
            _ds_pool,
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

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        None
    }

    fn render(
        &self,
        _ctx: &mut GfxContext,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw compute");

        let mut stats = RenderStats::default();

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
        stats.bind_pipeline(self.id);
        cmd.bind_resource(draw_bindings, &self.pipeline);
        stats.bind_resource(self.id);

        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);
        stats.draw(self.id, 0);

        cmd.end_label();

        stats.finish(self.id);

        Ok(())
    }
}
