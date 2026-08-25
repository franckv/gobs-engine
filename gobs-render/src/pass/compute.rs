use gobs_core::{ImageExtent2D, logger};
use gobs_render_graph::{FrameData, GraphResourceManager, PassMetaData};
use gobs_render_hal::{BindResource, BindingGroupType, Handle};

use crate::{GfxContext, RenderError};

pub struct ComputePassData {
    pub pipeline: Handle,
}

impl ComputePassData {
    pub fn new(pipeline: Handle) -> Self {
        Self { pipeline }
    }
}

pub struct ComputePass;

impl ComputePass {
    pub fn render(
        ctx: &mut GfxContext,
        pass_data: &mut ComputePassData,
        pass_metadata: &PassMetaData,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw compute");

        let cmd = &mut frame.command;

        cmd.begin_label("Draw compute");

        let mut resources = vec![];

        let mut draw_extent = ImageExtent2D::default();

        for name in pass_metadata.attachments.keys() {
            let image = resource_manager.image(name);
            draw_extent = ctx.get_image_extent(image);

            resources.push(image);
        }

        cmd.bind_pipeline(ctx, pass_data.pipeline);

        if !resources.is_empty() {
            // TODO: assume one ds
            let binding_layout = ctx
                .get_pipeline_descriptor_layout(pass_data.pipeline, &BindingGroupType::ComputeData)
                .ok_or(RenderError::InvalidData)?;

            let bind_resource = BindResource::with_resources(binding_layout.clone(), resources);
            cmd.bind_resource(ctx, pass_data.pipeline, &bind_resource);
        }

        // TODO: hardcoded
        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);

        cmd.end_label();

        Ok(())
    }
}
