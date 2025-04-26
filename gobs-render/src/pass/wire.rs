use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{
    BindingGroupType, Buffer, Command, CullMode, DescriptorStage, DescriptorType, DynamicStateElem,
    FrontFace, GfxCommand, GfxPipeline, GraphicsPipelineBuilder, ImageLayout, Pipeline,
    PolygonMode, Rect2D, Viewport,
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
    RenderError,
    batch::RenderBatch,
    context::Context,
    graph::ResourceManager,
    pass::{FrameData, PassId, PassType, RenderPass, RenderState},
    renderable::RenderObject,
    stats::RenderStats,
};

pub struct WirePass {
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

impl WirePass {
    pub fn new(ctx: &Context, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let vertex_flags = VertexFlag::POSITION;

        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_data_layout = UniformLayout::builder()
            .prop("view_proj", UniformProp::Mat4F)
            .build();

        let pipeline = GfxPipeline::graphics(name, &ctx.device)
            .vertex_shader("wire.vert.spv", "main")?
            .fragment_shader("wire.frag.spv", "main")?
            .pool_size(ctx.frames_in_flight)
            .push_constants(push_layout.size())
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .polygon_mode(PolygonMode::Line)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(Some(ctx.color_format), Some(ctx.depth_format))
            .depth_test_disable()
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx, uniform_data_layout.clone()))
            .collect();

        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Wire,
            attachments: vec![String::from("draw")],
            pipeline,
            vertex_flags,
            push_layout,
            frame_data,
            uniform_data_layout,
        }))
    }

    fn prepare_scene_data(&self, ctx: &Context, batch: &mut RenderBatch) {
        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[ctx.frame_id()]
                .uniform_buffer
                .write()
                .update(scene_data);
        }
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
            stats.bind(self.id);
            state.last_pipeline = self.pipeline.id();
        }
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
            stats.bind(self.id);
            state.scene_data_bound = true;
        }
    }

    fn bind_object_data(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        tracing::trace!("Bind push constants");

        if let Some(push_layout) = render_object.pass.push_layout() {
            state.object_data.clear();

            let world_matrix = render_object.transform.matrix();

            let material = render_object.mesh.material.clone().unwrap();
            let pipeline = material.pipeline();

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

            cmd.push_constants(&pipeline, &state.object_data);
        }

        if state.last_index_buffer != render_object.mesh.index_buffer.id()
            || state.last_indices_offset != render_object.mesh.indices_offset
        {
            cmd.bind_index_buffer(
                &render_object.mesh.index_buffer,
                render_object.mesh.indices_offset,
            );
            stats.bind(self.id);
            state.last_index_buffer = render_object.mesh.index_buffer.id();
            state.last_indices_offset = render_object.mesh.indices_offset;
        }
    }

    fn render_batch(&self, ctx: &Context, cmd: &GfxCommand, batch: &mut RenderBatch) {
        let mut render_state = RenderState::default();

        self.prepare_scene_data(ctx, batch);

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
            batch.render_stats.draw(self.id);
        }
    }
}

impl RenderPass for WirePass {
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
        ctx: &mut Context,
        cmd: &GfxCommand,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!("Draw wire");

        cmd.begin_label("Draw wire");

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
