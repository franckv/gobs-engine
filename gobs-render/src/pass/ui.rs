use std::sync::Arc;

use gobs_utils::timer::Timer;
use parking_lot::RwLock;

use gobs_core::entity::uniform::{UniformLayout, UniformProp, UniformPropData};
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType},
    image::{ImageExtent2D, ImageLayout},
    pipeline::{Pipeline, PipelineId},
};

use crate::{
    batch::RenderBatch,
    context::Context,
    geometry::VertexFlag,
    graph::{RenderError, ResourceManager},
    material::MaterialInstanceId,
    pass::{FrameData, PassId, PassType, RenderPass},
    CommandBuffer,
};

pub struct UiPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    push_layout: Arc<UniformLayout>,
    frame_data: Vec<FrameData>,
    frame_number: RwLock<usize>,
    _uniform_ds_pool: DescriptorSetPool,
    uniform_data_layout: Arc<UniformLayout>,
}

impl UiPass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let push_layout = UniformLayout::builder()
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let uniform_data_layout = UniformLayout::builder()
            .prop("screen_size", UniformProp::Vec2F)
            .build();

        let mut _uniform_ds_pool = DescriptorSetPool::new(
            ctx.device.clone(),
            uniform_descriptor_layout.clone(),
            ctx.frames_in_flight as u32,
        );

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| {
                FrameData::new(
                    ctx,
                    uniform_data_layout.clone(),
                    _uniform_ds_pool.allocate(),
                )
            })
            .collect();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Ui,
            attachments: vec![String::from("draw")],
            push_layout,
            frame_data,
            frame_number: RwLock::new(0),
            _uniform_ds_pool,
            uniform_data_layout,
        })
    }

    fn new_frame(&self, ctx: &Context) -> usize {
        let mut frame_number = self.frame_number.write();
        *frame_number += 1;
        (*frame_number - 1) % ctx.frames_in_flight
    }

    fn render_batch(&self, ctx: &Context, cmd: &CommandBuffer, batch: &mut RenderBatch) {
        let mut timer = Timer::new();

        let frame_id = self.new_frame(ctx);

        let mut last_material = MaterialInstanceId::nil();
        let mut last_pipeline = PipelineId::nil();

        let uniform_data_ds = &self.frame_data[frame_id].uniform_ds;

        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[frame_id]
                .uniform_buffer
                .write()
                .update(scene_data);
        }
        batch.render_stats.cpu_draw_update += timer.delta();

        let mut model_data = Vec::new();

        for render_object in &batch.render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            if render_object.material.is_none() {
                continue;
            }
            let material = render_object.material.clone().unwrap();
            let pipeline = material.pipeline();

            if last_material != material.id {
                if last_pipeline != pipeline.id {
                    cmd.bind_pipeline(&pipeline);
                    batch.render_stats.binds += 1;
                    last_pipeline = pipeline.id;
                }
                cmd.bind_descriptor_set(uniform_data_ds, 0, &pipeline);
                batch.render_stats.binds += 1;
                if let Some(material_ds) = &material.material_ds {
                    cmd.bind_descriptor_set(material_ds, 1, &pipeline);
                    batch.render_stats.binds += 1;
                }

                last_material = material.id;
            }
            batch.render_stats.cpu_draw_bind += timer.delta();

            if let Some(push_layout) = render_object.pass.push_layout() {
                model_data.clear();
                push_layout.copy_data(
                    &[UniformPropData::U64(
                        render_object
                            .model
                            .vertex_buffer
                            .address(ctx.device.clone())
                            + render_object.vertices_offset,
                    )],
                    &mut model_data,
                );

                cmd.push_constants(pipeline.layout.clone(), &model_data);
            }
            batch.render_stats.cpu_draw_push += timer.delta();

            cmd.bind_index_buffer::<u32>(
                &render_object.model.index_buffer,
                render_object.indices_offset,
            );
            batch.render_stats.binds += 1;
            batch.render_stats.cpu_draw_bind += timer.delta();

            cmd.draw_indexed(render_object.indices_len, 1);
            batch.render_stats.draws += 1;
            batch.render_stats.cpu_draw_submit += timer.delta();
        }
    }
}

impl RenderPass for UiPass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> super::PassType {
        self.ty
    }

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn pipeline(&self) -> Option<Arc<Pipeline>> {
        None
    }

    fn vertex_flags(&self) -> Option<VertexFlag> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        Some(self.push_layout.clone())
    }

    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>> {
        Some(self.uniform_data_layout.clone())
    }

    fn render(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        log::debug!("Draw UI");

        cmd.begin_label("Draw UI");

        let image_attach = &self.attachments[0];

        cmd.transition_image_layout(
            &mut resource_manager.image_write(image_attach),
            ImageLayout::Color,
        );

        cmd.begin_rendering(
            Some(&resource_manager.image_read(image_attach)),
            draw_extent,
            None,
            false,
            false,
            [0.; 4],
            1.,
        );

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        self.render_batch(ctx, cmd, batch);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
