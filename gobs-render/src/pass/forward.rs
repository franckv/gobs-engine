use std::sync::Arc;

use glam::Mat3;
use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::entity::uniform::{UniformData, UniformLayout, UniformProp, UniformPropData};
use gobs_utils::timer::Timer;
use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    image::{Image, ImageExtent2D, ImageLayout},
    pipeline::Pipeline,
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::RenderError,
    pass::{PassId, PassType, RenderPass},
    renderable::{RenderObject, RenderStats},
    resources::UniformBuffer,
    CommandBuffer,
};

struct FrameData {
    pub uniform_ds: DescriptorSet,
    pub uniform_buffer: RwLock<UniformBuffer>,
}

impl FrameData {
    pub fn new(
        ctx: &Context,
        uniform_layout: Arc<UniformLayout>,
        uniform_ds: DescriptorSet,
    ) -> Self {
        let uniform_buffer = UniformBuffer::new(
            ctx,
            uniform_ds.layout.clone(),
            uniform_layout.size(),
            ctx.allocator.clone(),
        );

        uniform_ds
            .update()
            .bind_buffer(&uniform_buffer.buffer, 0, uniform_buffer.buffer.size)
            .end();

        FrameData {
            uniform_ds,
            uniform_buffer: RwLock::new(uniform_buffer),
        }
    }
}

pub struct ForwardPass {
    id: PassId,
    name: String,
    ty: PassType,
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

    fn draw(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_list: &[RenderObject],
        scene_data: Option<UniformData>,
        render_stats: &mut RenderStats,
    ) {
        let timer = Timer::new();

        let frame_id = self.new_frame(ctx);

        let mut last_model = Uuid::nil();
        let mut last_material = Uuid::nil();
        let mut last_pipeline = Uuid::nil();

        let uniform_data_ds = &self.frame_data[frame_id].uniform_ds;

        if let Some(scene_data) = scene_data {
            self.frame_data[frame_id]
                .uniform_buffer
                .write()
                .update(&scene_data);
        }

        for render_object in render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            render_stats.add_object(render_object);
            let world_matrix = render_object.transform.matrix;
            let normal_matrix = Mat3::from_quat(render_object.transform.rotation);

            let material = &render_object.material;
            let pipeline = material.pipeline();

            if last_material != material.id {
                if last_pipeline != pipeline.id {
                    cmd.bind_pipeline(&pipeline);
                    render_stats.binds += 1;
                    last_pipeline = pipeline.id;
                }
                cmd.bind_descriptor_set(uniform_data_ds, 0, &pipeline);
                render_stats.binds += 1;
                if let Some(material_ds) = &material.material_ds {
                    cmd.bind_descriptor_set(material_ds, 1, &pipeline);
                    render_stats.binds += 1;
                }

                last_material = material.id;
            }

            if let Some(push_layout) = render_object.pass.push_layout() {
                // TODO: hardcoded
                let model_data = UniformData::new(
                    &push_layout,
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
                );
                cmd.push_constants(pipeline.layout.clone(), &model_data.raw());
            }

            if last_model != render_object.model.model.id {
                cmd.bind_index_buffer::<u32>(
                    &render_object.model.index_buffer,
                    render_object.indices_offset,
                );
                render_stats.binds += 1;
                last_model = render_object.model.model.id;
            }
            cmd.draw_indexed(render_object.indices_len, 1);
            render_stats.draws += 1;
        }

        render_stats.cpu_draw_time = timer.peek();
    }

    fn render(
        self: Arc<Self>,
        _ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &mut dyn FnMut(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError> {
        log::debug!("Draw forward");

        cmd.begin_label("Draw forward");

        cmd.transition_image_layout(&mut render_targets[0], ImageLayout::Color);
        cmd.transition_image_layout(&mut render_targets[1], ImageLayout::Depth);

        cmd.begin_rendering(
            &render_targets[0],
            draw_extent,
            Some(&render_targets[1]),
            false,
            [0.; 4],
            1.,
        );

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        draw_cmd(self, cmd);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
