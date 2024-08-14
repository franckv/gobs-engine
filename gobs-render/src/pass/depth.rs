use std::sync::Arc;

use uuid::Uuid;

use gobs_core::{
    entity::{
        camera::Camera,
        light::Light,
        uniform::{UniformLayout, UniformProp, UniformPropData},
    },
    Transform,
};
use gobs_gfx::{
    BindingGroupType, Buffer, Command, CompareOp, CullMode, DescriptorStage, DescriptorType,
    DynamicStateElem, FrontFace, ImageExtent2D, ImageLayout, Pipeline, PolygonMode, Rect2D,
    Viewport,
};
use gobs_utils::timer::Timer;

use crate::{
    batch::RenderBatch,
    context::Context,
    geometry::VertexFlag,
    graph::{RenderError, ResourceManager},
    pass::{FrameData, PassId, PassType, RenderPass},
    GfxCommand, GfxPipeline,
};

pub struct DepthPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    pipeline: Arc<GfxPipeline>,
    vertex_flags: VertexFlag,
    push_layout: Arc<UniformLayout>,
    frame_data: Vec<FrameData>,
    uniform_data_layout: Arc<UniformLayout>,
}

impl DepthPass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let vertex_flags = VertexFlag::POSITION;

        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_data_layout = UniformLayout::builder()
            .prop("view_proj", UniformProp::Mat4F)
            .build();

        let pipeline_builder = GfxPipeline::graphics(name, &ctx.device);

        let pipeline = pipeline_builder
            .vertex_shader("depth.vert.spv", "main")
            .pool_size(ctx.frames_in_flight)
            .push_constants(push_layout.size())
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::Vertex)
            .polygon_mode(PolygonMode::Fill)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(None, Some(ctx.depth_format))
            .depth_test_enable(true, CompareOp::Less)
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx, uniform_data_layout.clone()))
            .collect();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Depth,
            attachments: vec![String::from("depth")],
            pipeline,
            vertex_flags,
            push_layout,
            frame_data,
            uniform_data_layout,
        })
    }

    fn render_batch(&self, ctx: &Context, cmd: &GfxCommand, batch: &mut RenderBatch) {
        let mut timer = Timer::new();

        let frame_id = ctx.frame_id();

        let mut last_model = Uuid::nil();
        let mut last_offset = 0;

        cmd.bind_pipeline(&self.pipeline);
        batch.render_stats.binds += 1;
        batch.render_stats.cpu_draw_bind += timer.delta();

        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[frame_id]
                .uniform_buffer
                .write()
                .update(scene_data);
        }

        batch.render_stats.cpu_draw_update += timer.delta();

        let uniform_buffer = self.frame_data[frame_id].uniform_buffer.read();
        cmd.bind_resource_buffer(&uniform_buffer.buffer, &self.pipeline);

        batch.render_stats.binds += 1;
        batch.render_stats.cpu_draw_bind += timer.delta();

        let mut model_data = Vec::new();

        for render_object in &batch.render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            if render_object.material.is_none() {
                continue;
            }
            if render_object
                .material
                .clone()
                .unwrap()
                .material
                .blending_enabled
            {
                continue;
            }

            let world_matrix = render_object.transform.matrix();

            if let Some(push_layout) = self.push_layout() {
                model_data.clear();
                // TODO: hardcoded
                push_layout.copy_data(
                    &[
                        UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        UniformPropData::U64(
                            render_object.model.vertex_buffer.address(&ctx.device)
                                + render_object.vertices_offset,
                        ),
                    ],
                    &mut model_data,
                );

                cmd.push_constants(&self.pipeline, &model_data);
            }
            batch.render_stats.cpu_draw_push += timer.delta();

            if last_model != render_object.model.model.id
                || last_offset != render_object.indices_offset
            {
                cmd.bind_index_buffer(
                    &render_object.model.index_buffer,
                    render_object.indices_offset,
                );
                batch.render_stats.binds += 1;
                last_model = render_object.model.model.id;
                last_offset = render_object.indices_offset;
            }
            batch.render_stats.cpu_draw_bind += timer.delta();

            cmd.draw_indexed(render_object.indices_len, 1);
            batch.render_stats.draws += 1;
            batch.render_stats.cpu_draw_submit += timer.delta();
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

    fn color_clear(&self) -> bool {
        false
    }

    fn depth_clear(&self) -> bool {
        true
    }

    fn pipeline(&self) -> Option<Arc<GfxPipeline>> {
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

    fn get_uniform_data(
        &self,
        camera: &Camera,
        camera_transform: &Transform,
        _light: &Light,
        _light_transform: &Transform,
    ) -> Vec<u8> {
        self.uniform_data_layout.data(&[UniformPropData::Mat4F(
            camera
                .view_proj(camera_transform.translation())
                .to_cols_array_2d(),
        )])
    }

    fn render(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
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
            self.color_clear(),
            self.depth_clear(),
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
