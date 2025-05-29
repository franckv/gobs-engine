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
    geometry::VertexAttribute,
};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    graph::GraphResourceManager,
    pass::{PassFrameData, PassId, PassType, RenderPass, RenderState},
};

pub struct WirePass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    pipeline: Arc<GfxPipeline>,
    vertex_attributes: VertexAttribute,
    push_layout: Arc<UniformLayout>,
    frame_data: Vec<PassFrameData>,
    uniform_data_layout: Arc<UniformLayout>,
}

impl WirePass {
    pub fn new(ctx: &GfxContext, name: &str) -> Result<Arc<dyn RenderPass>, RenderError> {
        let vertex_attributes = VertexAttribute::POSITION;

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
            .map(|_| PassFrameData::new(ctx, uniform_data_layout.clone()))
            .collect();

        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Wire,
            attachments: vec![String::from("draw")],
            pipeline,
            vertex_attributes,
            push_layout,
            frame_data,
            uniform_data_layout,
        }))
    }

    fn prepare_scene_data(&self, frame: &PassFrameData, uniform_data: &[u8]) {
        frame.uniform_buffer.write().update(uniform_data);
    }

    fn should_render(&self, render_object: &RenderObject) -> bool {
        render_object.pass.id() == self.id
    }

    fn bind_pipeline(
        &self,
        cmd: &GfxCommand,
        state: &mut RenderState,
        _render_object: &RenderObject,
    ) {
        if state.last_pipeline != self.pipeline.id() {
            cmd.bind_pipeline(&self.pipeline);
            state.last_pipeline = self.pipeline.id();
        }
    }

    fn bind_scene_data(
        &self,
        frame: &PassFrameData,
        cmd: &GfxCommand,
        state: &mut RenderState,
        _render_object: &RenderObject,
    ) {
        if !state.scene_data_bound {
            let uniform_buffer = frame.uniform_buffer.read();

            cmd.bind_resource_buffer(&uniform_buffer.buffer, &self.pipeline);
            state.scene_data_bound = true;
        }
    }

    fn bind_object_data(
        &self,
        ctx: &GfxContext,
        cmd: &GfxCommand,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        tracing::trace!(target: "render", "Bind push constants");

        if let Some(push_layout) = render_object.pass.push_layout() {
            state.object_data.clear();

            let world_matrix = render_object.transform.matrix();

            let pipeline = render_object.pipeline.clone().unwrap();

            // TODO: hardcoded
            push_layout.copy_data(
                &[
                    UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                    UniformPropData::U64(
                        render_object.vertex_buffer.address(&ctx.device)
                            + render_object.vertices_offset,
                    ),
                ],
                &mut state.object_data,
            );

            cmd.push_constants(&pipeline, &state.object_data);
        }

        if state.last_index_buffer != render_object.index_buffer.id()
            || state.last_indices_offset != render_object.indices_offset
        {
            cmd.bind_index_buffer(&render_object.index_buffer, render_object.indices_offset);
            state.last_index_buffer = render_object.index_buffer.id();
            state.last_indices_offset = render_object.indices_offset;
        }
    }

    fn render_batch(
        &self,
        ctx: &GfxContext,
        frame: &PassFrameData,
        cmd: &GfxCommand,
        render_list: &[RenderObject],
        uniform_data: Option<&[u8]>,
    ) {
        let mut render_state = RenderState::default();

        if let Some(uniform_data) = uniform_data {
            self.prepare_scene_data(frame, uniform_data);
        }

        for render_object in render_list {
            if !self.should_render(render_object) {
                continue;
            }

            self.bind_pipeline(cmd, &mut render_state, render_object);

            self.bind_scene_data(frame, cmd, &mut render_state, render_object);

            self.bind_object_data(ctx, cmd, &mut render_state, render_object);

            cmd.draw_indexed(render_object.indices_len, 1);
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

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        Some(self.vertex_attributes)
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
        ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        render_list: &[RenderObject],
        uniform_data: Option<&[u8]>,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Draw wire");

        let cmd = &frame.command;

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

        let pass_frame = &self.frame_data[frame.id];

        self.render_batch(ctx, pass_frame, cmd, render_list, uniform_data);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
