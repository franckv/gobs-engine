use std::sync::Arc;

use gobs_core::entity::uniform::UniformLayout;
use gobs_utils::load;
use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    image::{ImageExtent2D, ImageLayout},
    pipeline::{Pipeline, PipelineLayout, Shader, ShaderType},
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::{RenderError, ResourceManager},
    pass::{PassId, PassType, RenderPass},
    renderable::RenderBatch,
    CommandBuffer,
};

pub struct ComputePass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    _draw_ds_pool: DescriptorSetPool,
    pub draw_ds: DescriptorSet,
    pub pipeline: Arc<Pipeline>,
}

impl ComputePass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let _draw_ds_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone());
        let mut _draw_ds_pool =
            DescriptorSetPool::new(ctx.device.clone(), _draw_ds_layout.clone(), 10);
        let draw_ds = _draw_ds_pool.allocate();

        let compute_file = load::get_asset_dir("sky.comp.spv", load::AssetType::SHADER).unwrap();
        let compute_shader =
            Shader::from_file(compute_file, ctx.device.clone(), ShaderType::Compute);

        let pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), &[_draw_ds_layout.clone()], 0);
        let pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .compute_shader("main", compute_shader)
            .build();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Compute,
            attachments: vec![String::from("draw")],
            _draw_ds_pool,
            draw_ds,
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

    fn pipeline(&self) -> Option<Arc<Pipeline>> {
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

    fn render(
        &self,
        _ctx: &Context,
        cmd: &CommandBuffer,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        log::debug!("Draw compute");
        cmd.begin_label("Draw compute");

        let draw_attach = &self.attachments[0];

        self.draw_ds
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
        cmd.bind_descriptor_set(&self.draw_ds, 0, &self.pipeline);
        batch.stats_mut().binds += 2;

        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);
        batch.stats_mut().draws += 1;

        cmd.end_label();

        Ok(())
    }
}