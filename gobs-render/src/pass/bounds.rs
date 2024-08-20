use std::sync::Arc;

use gobs_core::{utils::timer::Timer, ImageExtent2D, Transform};
use gobs_gfx::{
    BindingGroupType, Buffer, Command, CullMode, DescriptorStage, DescriptorType, DynamicStateElem,
    FrontFace, ImageLayout, Pipeline, PolygonMode, Rect2D, Viewport,
};
use gobs_resource::{
    entity::{
        camera::Camera,
        light::Light,
        uniform::{UniformLayout, UniformProp, UniformPropData},
    },
    geometry::VertexFlag,
};

use crate::{
    batch::RenderBatch,
    context::Context,
    graph::{RenderError, ResourceManager},
    pass::{FrameData, PassId, PassType, RenderPass},
    renderable::RenderObject,
    stats::RenderStats,
    GfxCommand, GfxPipeline,
};

use super::RenderState;

pub struct BoundsPass {
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

impl BoundsPass {
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
            .vertex_shader("wire.vert.spv", "main")
            .fragment_shader("wire.frag.spv", "main")
            .pool_size(ctx.frames_in_flight)
            .push_constants(push_layout.size())
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .polygon_mode(PolygonMode::Line)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(Some(ctx.color_format), Some(ctx.depth_format))
            .depth_test_disable()
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx, uniform_data_layout.clone()))
            .collect();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Bounds,
            attachments: vec![String::from("draw")],
            pipeline,
            vertex_flags,
            push_layout,
            frame_data,
            uniform_data_layout,
        })
    }

    fn prepare_scene_data(&self, ctx: &Context, state: &mut RenderState, batch: &mut RenderBatch) {
        batch.render_stats.cpu_draw_update += state.timer.delta();

        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[ctx.frame_id()]
                .uniform_buffer
                .write()
                .update(scene_data);
        }

        batch.render_stats.cpu_draw_update += state.timer.delta();
    }

    fn should_render(&self, render_object: &RenderObject) -> bool {
        render_object.pass.id() == self.id
    }

    fn bind_pipeline(
        &self,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        _render_object: &RenderObject,
    ) {
        if state.last_pipeline != self.pipeline.id() {
            cmd.bind_pipeline(&self.pipeline);
            stats.binds += 1;
            state.last_pipeline = self.pipeline.id();
        }
        stats.cpu_draw_bind += state.timer.delta();
    }

    fn bind_scene_data(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        _render_object: &RenderObject,
    ) {
        if !state.scene_data_bound {
            let uniform_buffer = self.frame_data[ctx.frame_id()].uniform_buffer.read();

            cmd.bind_resource_buffer(&uniform_buffer.buffer, &self.pipeline);
            stats.binds += 1;
            state.scene_data_bound = true;
        }
        stats.cpu_draw_bind += state.timer.delta();
    }

    fn bind_object_data(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        log::trace!("Bind push constants");

        if let Some(push_layout) = self.push_layout() {
            state.object_data.clear();

            let world_matrix = render_object.transform.matrix();

            // TODO: hardcoded
            push_layout.copy_data(
                &[
                    UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                    UniformPropData::U64(
                        render_object.mesh.vertex_buffer.address(&ctx.device)
                            + render_object.mesh.vertices_offset,
                    ),
                ],
                &mut state.object_data,
            );

            cmd.push_constants(&self.pipeline, &state.object_data);
        }
        stats.cpu_draw_push += state.timer.delta();

        if state.last_index_buffer != render_object.mesh.index_buffer.id()
            || state.last_indices_offset != render_object.mesh.indices_offset
        {
            cmd.bind_index_buffer(
                &render_object.mesh.index_buffer,
                render_object.mesh.indices_offset,
            );
            stats.binds += 1;
            state.last_index_buffer = render_object.mesh.index_buffer.id();
            state.last_indices_offset = render_object.mesh.indices_offset;
        }
        stats.cpu_draw_bind += state.timer.delta();
    }

    fn render_batch(&self, ctx: &Context, cmd: &GfxCommand, batch: &mut RenderBatch) {
        let mut timer = Timer::new();

        let mut render_state = RenderState::new();

        self.prepare_scene_data(ctx, &mut render_state, batch);

        for render_object in &batch.render_list {
            if !self.should_render(render_object) {
                continue;
            }

            self.bind_pipeline(
                cmd,
                &mut batch.render_stats,
                &mut render_state,
                render_object,
            );

            self.bind_scene_data(
                ctx,
                cmd,
                &mut batch.render_stats,
                &mut render_state,
                render_object,
            );

            self.bind_object_data(
                ctx,
                cmd,
                &mut batch.render_stats,
                &mut render_state,
                render_object,
            );

            cmd.draw_indexed(render_object.mesh.indices_len, 1);
            batch.render_stats.draws += 1;
            batch.render_stats.cpu_draw_submit += timer.delta();
        }
    }
}

impl RenderPass for BoundsPass {
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
        false
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
        log::debug!("Draw bounds");

        cmd.begin_label("Draw bounds");

        let draw_attach = &self.attachments[0];

        cmd.transition_image_layout(
            &mut resource_manager.image_write(draw_attach),
            ImageLayout::Color,
        );

        cmd.begin_rendering(
            Some(&resource_manager.image_read(draw_attach)),
            draw_extent,
            None,
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
