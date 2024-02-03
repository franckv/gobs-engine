use std::sync::Arc;

use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    image::{Image, ImageExtent2D, ImageLayout},
    pipeline::{Pipeline, PipelineLayout, Shader, ShaderType},
};

use crate::{context::Context, graph::RenderError, CommandBuffer};

use super::{PassType, RenderPass};

const SHADER_DIR: &str = "examples/shaders";

pub struct ComputePass {
    name: String,
    ty: PassType,
    _draw_ds_pool: DescriptorSetPool,
    _draw_ds_layout: Arc<DescriptorSetLayout>,
    pub draw_ds: DescriptorSet,
    pub bg_pipeline: Pipeline,
    _bg_pipeline_layout: Arc<PipelineLayout>,
}

impl ComputePass {
    pub fn new(ctx: &Context, name: &str, render_target: &Image) -> Self {
        let _draw_ds_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone());
        let mut _draw_ds_pool =
            DescriptorSetPool::new(ctx.device.clone(), _draw_ds_layout.clone(), 10);
        let draw_ds = _draw_ds_pool.allocate();

        draw_ds
            .update()
            .bind_image(render_target, ImageLayout::General)
            .end();

        let compute_shader = Shader::from_file(
            &format!("{}/sky.comp.spv", SHADER_DIR),
            ctx.device.clone(),
            ShaderType::Compute,
        );

        let _bg_pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), &[_draw_ds_layout.clone()], 0);
        let bg_pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(_bg_pipeline_layout.clone())
            .compute_shader("main", compute_shader)
            .build();

        Self {
            name: name.to_string(),
            ty: PassType::Compute,
            _draw_ds_pool,
            _draw_ds_layout,
            draw_ds,
            bg_pipeline,
            _bg_pipeline_layout,
        }
    }
}

impl RenderPass for ComputePass {
    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn render<F>(
        &self,
        _ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        _draw_extent: ImageExtent2D,
        draw_cmd: &F,
    ) -> Result<(), RenderError>
    where
        F: Fn(PassType, &str, &CommandBuffer),
    {
        log::debug!("Draw compute");
        cmd.begin_label("Draw compute");

        cmd.transition_image_layout(&mut render_targets[0], ImageLayout::General);

        cmd.bind_pipeline(&self.bg_pipeline);
        cmd.bind_descriptor_set(&self.draw_ds, 0, &self.bg_pipeline);

        draw_cmd(self.ty, &self.name, cmd);

        cmd.end_label();

        Ok(())
    }
}
