use std::sync::Arc;

use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{
    BindResource, BindingGroupLayout, BindingGroupType, DescriptorStage, DescriptorType, Handle,
    ImageLayout,
};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject, SceneData,
    graph::GraphResourceManager,
    pass::{PassId, PassType, RenderPass},
};

pub struct ComputePass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    pub pipeline: Handle,
    binding_layout: BindingGroupLayout,
}

impl ComputePass {
    pub fn new(ctx: &mut GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let pipeline_builder = ctx.hal.create_compute_pipeline(name);

        let binding_layout = BindingGroupLayout::new(BindingGroupType::ComputeData)
            .add_binding(DescriptorType::StorageImage, DescriptorStage::Compute);

        let pipeline = pipeline_builder
            .shader("sky.comp.spv", "main")
            .binding_group(binding_layout.clone())
            .build(ctx.hal.as_mut());

        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Compute,
            attachments: vec![String::from("draw")],
            pipeline,
            binding_layout,
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

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        _render_list: &[RenderObject],
        _scene_data: &SceneData,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw compute");

        let cmd = &frame.command;

        cmd.begin_label("Draw compute");

        let draw_attach = &self.attachments[0];

        let draw_image = resource_manager.image(draw_attach);

        // draw_bindings
        //     .update()
        //     .bind_image(&resource_manager.image(draw_attach), ImageLayout::General)
        //     .end();
        //

        cmd.transition_image_layout(
            ctx.hal.as_mut(),
            resource_manager.image(draw_attach),
            ImageLayout::General,
        );

        cmd.bind_pipeline(ctx.hal.as_ref(), self.pipeline);

        let bind_resource = BindResource::new(self.binding_layout.clone(), vec![draw_image]);
        cmd.bind_resource(ctx.hal.as_mut(), self.pipeline, &bind_resource);

        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);

        cmd.end_label();

        Ok(())
    }
}
