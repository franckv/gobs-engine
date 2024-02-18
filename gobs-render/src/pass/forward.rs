use std::sync::Arc;

use glam::Mat3;
use gobs_utils::timer::Timer;
use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::entity::uniform::{UniformLayout, UniformProp, UniformPropData};
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType},
    image::{ImageExtent2D, ImageLayout},
    pipeline::Pipeline,
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::{RenderError, ResourceManager},
    pass::{FrameData, PassId, PassType, RenderPass},
    renderable::RenderBatch,
    CommandBuffer,
};

pub struct ForwardPass {
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

impl ForwardPass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("normal_matrix", UniformProp::Mat3F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let uniform_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
            .prop("light_direction", UniformProp::Vec3F)
            .prop("light_color", UniformProp::Vec4F)
            .prop("ambient_color", UniformProp::Vec4F)
            .build();

        let mut uniform_ds_pool = DescriptorSetPool::new(
            ctx.device.clone(),
            uniform_descriptor_layout.clone(),
            ctx.frames_in_flight as u32,
        );

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx, uniform_data_layout.clone(), uniform_ds_pool.allocate()))
            .collect();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Forward,
            attachments: vec![String::from("draw"), String::from("depth")],
            push_layout,
            frame_data,
            frame_number: RwLock::new(0),
            _uniform_ds_pool: uniform_ds_pool,
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

        let mut last_model = Uuid::nil();
        let mut last_material = Uuid::nil();
        let mut last_pipeline = Uuid::nil();

        let uniform_data_ds = &self.frame_data[frame_id].uniform_ds;

        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[frame_id]
                .uniform_buffer
                .write()
                .update(scene_data);
        }

        let mut model_data = Vec::new();

        for render_object in &batch.render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            let world_matrix = render_object.transform.matrix;
            let normal_matrix = Mat3::from_quat(render_object.transform.rotation);

            let material = &render_object.material;
            let pipeline = material.pipeline();

            if last_material != material.id {
                if last_pipeline != pipeline.id {
                    cmd.bind_pipeline(&pipeline);
                    cmd.bind_descriptor_set(uniform_data_ds, 0, &pipeline);
                    batch.render_stats.binds += 2;
                    last_pipeline = pipeline.id;
                }

                if let Some(material_ds) = &material.material_ds {
                    cmd.bind_descriptor_set(material_ds, 1, &pipeline);
                    batch.render_stats.binds += 1;
                }

                last_material = material.id;
            }

            if let Some(push_layout) = render_object.pass.push_layout() {
                model_data.clear();
                // TODO: hardcoded
                push_layout.data_buf(
                    &[
                        UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        UniformPropData::Mat3F(normal_matrix.to_cols_array_2d()),
                        UniformPropData::U64(
                            render_object
                                .model
                                .vertex_buffer
                                .address(ctx.device.clone()),
                        ),
                    ],
                    &mut model_data,
                );

                batch.render_stats.cpu_draw_pre += timer.delta();

                cmd.push_constants(pipeline.layout.clone(), &model_data);
            }

            batch.render_stats.cpu_draw_mid += timer.delta();

            if last_model != render_object.model.model.id {
                cmd.bind_index_buffer::<u32>(
                    &render_object.model.index_buffer,
                    render_object.indices_offset,
                );
                batch.render_stats.binds += 1;
                last_model = render_object.model.model.id;
            }
            cmd.draw_indexed(render_object.indices_len, 1);
            batch.render_stats.draws += 1;

            batch.render_stats.cpu_draw_post += timer.delta();
        }
    }
}

impl RenderPass for ForwardPass {
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
        log::debug!("Draw forward");

        cmd.begin_label("Draw forward");

        let draw_attach = &self.attachments[0];
        let depth_attach = &self.attachments[1];

        cmd.transition_image_layout(
            &mut resource_manager.image_write(draw_attach),
            ImageLayout::Color,
        );
        cmd.transition_image_layout(
            &mut resource_manager.image_write(depth_attach),
            ImageLayout::Depth,
        );

        cmd.begin_rendering(
            Some(&resource_manager.image_read(draw_attach)),
            draw_extent,
            Some(&resource_manager.image_read(depth_attach)),
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
