use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{Buffer, Command, GfxCommand, GfxPipeline, ImageLayout, Pipeline};
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

pub struct UiPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    color_clear: bool,
    push_layout: Arc<UniformLayout>,
    frame_data: Vec<PassFrameData>,
    uniform_data_layout: Arc<UniformLayout>,
}

impl UiPass {
    pub fn new(
        ctx: &GfxContext,
        name: &str,
        color_clear: bool,
    ) -> Result<Arc<dyn RenderPass>, RenderError> {
        let push_layout = UniformLayout::builder()
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_data_layout = UniformLayout::builder()
            .prop("screen_size", UniformProp::Vec2F)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| PassFrameData::new(ctx, uniform_data_layout.clone()))
            .collect();

        Ok(Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Ui,
            attachments: vec![String::from("draw")],
            color_clear,
            push_layout,
            frame_data,
            uniform_data_layout,
        }))
    }

    fn prepare_scene_data(&self, frame: &PassFrameData, uniform_data: &[u8]) {
        frame.uniform_buffer.write().update(uniform_data);
    }

    fn should_render(&self, render_object: &RenderObject) -> bool {
        render_object.pass.id() == self.id && render_object.pipeline.is_some()
    }

    fn bind_pipeline(
        &self,
        cmd: &GfxCommand,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        let pipeline = render_object.pipeline.clone().unwrap();

        if state.last_pipeline != pipeline.id() {
            tracing::trace!(target: "render", "Bind pipeline {}", pipeline.id());

            cmd.bind_pipeline(&pipeline);

            state.last_pipeline = pipeline.id();
        }
    }

    fn bind_resources(&self, cmd: &GfxCommand, render_object: &RenderObject) {
        for bind_group in &render_object.bind_groups {
            cmd.bind_resource(bind_group);
        }
    }

    fn bind_scene_data(
        &self,
        frame: &PassFrameData,
        cmd: &GfxCommand,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        if !state.scene_data_bound {
            let pipeline = render_object.pipeline.clone().unwrap();
            let uniform_buffer = frame.uniform_buffer.read();

            cmd.bind_resource_buffer(&uniform_buffer.buffer, &pipeline);
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

            let pipeline = render_object.pipeline.clone().unwrap();

            // TODO: hardcoded
            push_layout.copy_data(
                &[UniformPropData::U64(
                    render_object.vertex_buffer.address(&ctx.device)
                        + render_object.vertices_offset,
                )],
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
        frame_id: usize,
        cmd: &GfxCommand,
        render_list: &[RenderObject],
        uniform_data: Option<&[u8]>,
    ) {
        let mut render_state = RenderState::default();

        let frame = &self.frame_data[frame_id];

        if let Some(uniform_data) = uniform_data {
            self.prepare_scene_data(frame, uniform_data);
        }

        for render_object in render_list {
            if !self.should_render(render_object) {
                continue;
            }

            self.bind_pipeline(cmd, &mut render_state, render_object);

            self.bind_scene_data(frame, cmd, &mut render_state, render_object);

            self.bind_resources(cmd, render_object);

            self.bind_object_data(ctx, cmd, &mut render_state, render_object);

            cmd.draw_indexed(render_object.indices_len, 1);
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

    fn ty(&self) -> PassType {
        self.ty
    }

    fn attachments(&self) -> &[String] {
        &self.attachments
    }

    fn color_clear(&self) -> bool {
        self.color_clear
    }

    fn depth_clear(&self) -> bool {
        false
    }

    fn pipeline(&self) -> Option<Arc<GfxPipeline>> {
        None
    }

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        Some(self.push_layout.clone())
    }

    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>> {
        Some(self.uniform_data_layout.clone())
    }

    fn get_uniform_data(
        &self,
        _camera: &Camera,
        _camera_transform: &Transform,
        _light: &Light,
        _light_transform: &Transform,
    ) -> Vec<u8> {
        vec![]
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
        tracing::debug!(target: "render", "Draw UI");

        let cmd = &frame.command;

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
            self.color_clear(),
            self.depth_clear(),
            [0.; 4],
            1.,
        );

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        self.render_batch(ctx, frame.id, cmd, render_list, uniform_data);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
