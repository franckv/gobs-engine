use std::collections::HashMap;

use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{BindResource, BindingGroupLayout, Handle, SceneData};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    graph::GraphResourceManager,
    pass::{Attachment, AttachmentType, PassId, PassType, RenderPass},
};

pub struct ComputePass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: HashMap<String, Attachment>,
    image_attachments: Vec<String>,
    pub pipeline: Handle,
    binding_layout: Vec<BindingGroupLayout>,
}

impl ComputePass {
    pub fn new(name: &str, pipeline: Handle, binding_layout: Vec<BindingGroupLayout>) -> Self {
        Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Compute,
            attachments: Default::default(),
            image_attachments: vec![],
            pipeline,
            binding_layout,
        }
    }

    pub fn add_attachment(&mut self, name: &str, attachment: Attachment) {
        match attachment.ty {
            AttachmentType::ImageStorage => self.image_attachments.push(name.to_string()),
            _ => todo!(),
        }

        self.attachments.insert(name.to_string(), attachment);
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

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw compute");

        let cmd = &frame.command;

        cmd.begin_label("Draw compute");

        let mut resources = vec![];

        let mut draw_extent = ImageExtent2D::default();

        for (name, attachment) in &self.attachments {
            cmd.transition_image_layout(
                ctx.hal.as_mut(),
                resource_manager.image(name),
                attachment.layout,
            );

            let image = resource_manager.image(name);
            draw_extent = ctx.hal.get_image_extent(image);

            resources.push(image);
        }

        cmd.bind_pipeline(ctx.hal.as_ref(), self.pipeline);

        if !resources.is_empty() {
            // TODO: assume one ds
            let bind_resource = BindResource::new(self.binding_layout[0].clone(), resources);
            cmd.bind_resource(ctx.hal.as_mut(), self.pipeline, &bind_resource);
        }

        // TODO: hardcoded
        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);

        cmd.end_label();

        Ok(())
    }
}
