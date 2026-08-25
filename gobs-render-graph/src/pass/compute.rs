use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{BindResource, BindingGroupType, Handle};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::SceneData,
    graph::GraphResourceManager,
    pass::{PassId, RenderPass, metadata::PassMetaData},
};

pub struct ComputePass {
    pub metadata: PassMetaData,
    pub pipeline: Handle,
}

impl ComputePass {
    pub fn new(metadata: PassMetaData, pipeline: Handle) -> Self {
        Self { metadata, pipeline }
    }
}

impl RenderPass for ComputePass {
    fn id(&self) -> PassId {
        self.metadata.id
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn metadata(&self) -> &PassMetaData {
        &self.metadata
    }

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw compute");

        let cmd = &mut frame.command;

        cmd.begin_label("Draw compute");

        let mut resources = vec![];

        let mut draw_extent = ImageExtent2D::default();

        for name in self.metadata.attachments.keys() {
            let image = resource_manager.image(name);
            draw_extent = ctx.hal().get_image_extent(image);

            resources.push(image);
        }

        cmd.bind_pipeline(ctx.hal(), self.pipeline);

        if !resources.is_empty() {
            // TODO: assume one ds
            let binding_layout = ctx
                .hal()
                .get_pipeline_descriptor_layout(self.pipeline, &BindingGroupType::ComputeData)
                .ok_or(RenderError::InvalidData)?;

            let bind_resource = BindResource::with_resources(binding_layout.clone(), resources);
            cmd.bind_resource(ctx.hal_mut(), self.pipeline, &bind_resource);
        }

        // TODO: hardcoded
        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);

        cmd.end_label();

        Ok(())
    }
}
