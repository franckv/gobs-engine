use std::sync::Arc;

use glam::Mat3;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{Buffer, Command, ImageLayout, Pipeline};
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

pub struct ForwardPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: Vec<String>,
    color_clear: bool,
    depth_clear: bool,
    push_layout: Arc<UniformLayout>,
    frame_data: Vec<FrameData>,
    uniform_data_layout: Arc<UniformLayout>,
}

impl ForwardPass {
    pub fn new(
        ctx: &Context,
        name: &str,
        color_clear: bool,
        depth_clear: bool,
    ) -> Arc<dyn RenderPass> {
        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("normal_matrix", UniformProp::Mat3F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let uniform_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
            .prop("light_direction", UniformProp::Vec3F)
            .prop("light_color", UniformProp::Vec4F)
            .prop("ambient_color", UniformProp::Vec4F)
            .build();

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx, uniform_data_layout.clone()))
            .collect();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Forward,
            attachments: vec![String::from("draw"), String::from("depth")],
            color_clear,
            depth_clear,
            push_layout,
            frame_data,
            uniform_data_layout,
        })
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
        render_object.pass.id() == self.id && render_object.mesh.material.is_some()
    }

    fn bind_pipeline(
        &self,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        let material = render_object.mesh.material.clone().unwrap();
        let pipeline = material.pipeline();

        if state.last_pipeline != pipeline.id() {
            tracing::trace!("Bind pipeline {}", pipeline.id());

            cmd.bind_pipeline(&pipeline);
            stats.bind(self.id);

            state.last_pipeline = pipeline.id();
        }
    }

    fn bind_material(
        &self,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        if let Some(material) = &render_object.mesh.material {
            if state.last_material != material.id {
                if let Some(material_binding) = &render_object.mesh.material_binding {
                    tracing::trace!("Bind material {}", material.id);
                    tracing::trace!("Transparent: {}", material.material.blending_enabled);
                    cmd.bind_resource(material_binding);
                    stats.bind(self.id);
                }

                state.last_material = material.id;
            }
        }
    }

    fn bind_scene_data(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
        stats: &mut RenderStats,
        state: &mut RenderState,
        render_object: &RenderObject,
    ) {
        if !state.scene_data_bound {
            let material = render_object.mesh.material.clone().unwrap();
            let pipeline = material.pipeline();
            let uniform_buffer = self.frame_data[ctx.frame_id()].uniform_buffer.read();

            cmd.bind_resource_buffer(&uniform_buffer.buffer, &pipeline);
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
            let normal_matrix = Mat3::from_quat(render_object.transform.rotation());

            let material = render_object.mesh.material.clone().unwrap();
            let pipeline = material.pipeline();

            // TODO: hardcoded
            push_layout.copy_data(
                &[
                    UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                    UniformPropData::Mat3F(normal_matrix.to_cols_array_2d()),
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

            self.bind_material(
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

    fn color_clear(&self) -> bool {
        self.color_clear
    }

    fn depth_clear(&self) -> bool {
        self.depth_clear
    }

    fn pipeline(&self) -> Option<Arc<GfxPipeline>> {
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

    fn get_uniform_data(
        &self,
        camera: &Camera,
        camera_transform: &Transform,
        light: &Light,
        light_transform: &Transform,
    ) -> Vec<u8> {
        self.uniform_data_layout.data(&[
            UniformPropData::Vec3F(camera_transform.translation().into()),
            UniformPropData::Mat4F(
                camera
                    .view_proj(camera_transform.translation())
                    .to_cols_array_2d(),
            ),
            UniformPropData::Vec3F(light_transform.translation().normalize().into()),
            UniformPropData::Vec4F(light.colour.into()),
            UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]),
        ])
    }

    fn render(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!("Draw forward");

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
