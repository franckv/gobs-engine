use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::entity::uniform::{UniformLayout, UniformProp, UniformPropData};
use gobs_utils::{load, timer::Timer};
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType},
    image::{ImageExtent2D, ImageLayout},
    pipeline::{
        CompareOp, CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout, PolygonMode,
        Rect2D, Shader, ShaderType, Viewport,
    },
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::{RenderError, ResourceManager},
    pass::{FrameData, PassId, PassType, RenderPass},
    renderable::RenderBatch,
    CommandBuffer,
};

pub struct DepthPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    pipeline: Arc<Pipeline>,
    vertex_flags: VertexFlag,
    push_layout: Arc<UniformLayout>,
    frame_data: Vec<FrameData>,
    frame_number: RwLock<usize>,
    _uniform_ds_pool: DescriptorSetPool,
    uniform_data_layout: Arc<UniformLayout>,
}

impl DepthPass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let vertex_flags = VertexFlag::POSITION;

        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::Vertex)
            .build(ctx.device.clone());

        let uniform_data_layout = UniformLayout::builder()
            .prop("view_proj", UniformProp::Mat4F)
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

        let ds_layouts = vec![uniform_descriptor_layout.clone()];

        let pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), &ds_layouts, push_layout.size());

        let vertex_file = load::get_asset_dir("depth.vert.spv", load::AssetType::SHADER).unwrap();
        let vertex_shader = Shader::from_file(vertex_file, ctx.device.clone(), ShaderType::Vertex);

        let pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .polygon_mode(PolygonMode::Fill)
            .vertex_shader("main", vertex_shader)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(None, Some(ctx.depth_format))
            .depth_test_enable(true, CompareOp::Less)
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Depth,
            attachments: vec![String::from("depth")],
            pipeline,
            vertex_flags,
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

        let mut last_model = Uuid::nil();
        let mut last_offset = 0;

        cmd.bind_pipeline(&self.pipeline);
        batch.render_stats.binds += 1;

        let uniform_data_ds = &self.frame_data[frame_id].uniform_ds;

        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[frame_id]
                .uniform_buffer
                .write()
                .update(scene_data);
        }

        cmd.bind_descriptor_set(uniform_data_ds, 0, &self.pipeline);
        batch.render_stats.binds += 1;

        let mut model_data = Vec::new();

        for render_object in &batch.render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            if render_object.material.material.blending_enabled {
                continue;
            }
            let world_matrix = render_object.transform.matrix;

            if let Some(push_layout) = self.push_layout() {
                model_data.clear();
                // TODO: hardcoded
                push_layout.copy_data(
                    &[
                        UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        UniformPropData::U64(
                            render_object
                                .model
                                .vertex_buffer
                                .address(ctx.device.clone())
                                + render_object.vertices_offset,
                        ),
                    ],
                    &mut model_data,
                );

                batch.render_stats.cpu_draw_pre += timer.delta();

                cmd.push_constants(self.pipeline.layout.clone(), &model_data);
            }

            batch.render_stats.cpu_draw_mid += timer.delta();

            if last_model != render_object.model.model.id
                || last_offset != render_object.indices_offset
            {
                cmd.bind_index_buffer::<u32>(
                    &render_object.model.index_buffer,
                    render_object.indices_offset,
                );
                batch.render_stats.binds += 1;
                last_model = render_object.model.model.id;
                last_offset = render_object.indices_offset;
            }
            cmd.draw_indexed(render_object.indices_len, 1);
            batch.render_stats.draws += 1;

            batch.render_stats.cpu_draw_post += timer.delta();
        }
    }
}

impl RenderPass for DepthPass {
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
        Some(self.pipeline.clone())
    }

    fn vertex_flags(&self) -> Option<VertexFlag> {
        Some(self.vertex_flags)
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
        log::debug!("Draw depth");

        cmd.begin_label("Draw depth");

        let depth_attach = &self.attachments[0];

        cmd.transition_image_layout(
            &mut resource_manager.image_write(depth_attach),
            ImageLayout::Depth,
        );

        cmd.begin_rendering(
            None,
            draw_extent,
            Some(&resource_manager.image_read(depth_attach)),
            false,
            true,
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
