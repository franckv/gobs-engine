use std::sync::Arc;

use glam::Mat3;
use uuid::Uuid;

use gobs_core::{utils::timer::Timer, ImageExtent2D, Transform};
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
    GfxCommand, GfxPipeline,
};

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

    fn render_batch(&self, ctx: &Context, cmd: &GfxCommand, batch: &mut RenderBatch) {
        let mut timer = Timer::new();

        let frame_id = ctx.frame_id();

        let mut last_buffer = Uuid::nil();
        let mut last_material = Uuid::nil();
        let mut last_pipeline = Uuid::nil();
        let mut last_offset = 0;

        if let Some(scene_data) = batch.scene_data(self.id) {
            self.frame_data[frame_id]
                .uniform_buffer
                .write()
                .update(scene_data);
        }

        let uniform_buffer = self.frame_data[frame_id].uniform_buffer.read();

        batch.render_stats.cpu_draw_update += timer.delta();

        let mut model_data = Vec::new();

        for render_object in &batch.render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            if render_object.mesh.material.is_none() {
                continue;
            }
            let world_matrix = render_object.transform.matrix();
            let normal_matrix = Mat3::from_quat(render_object.transform.rotation());

            let material = &render_object.mesh.material.clone().unwrap();
            let pipeline = material.pipeline();

            if last_material != material.id {
                if last_pipeline != pipeline.id() {
                    log::trace!("Transparent: {}", material.material.blending_enabled);
                    log::trace!("Bind pipeline {}", pipeline.id());

                    cmd.bind_pipeline(&pipeline);
                    cmd.bind_resource_buffer(&uniform_buffer.buffer, &pipeline);

                    batch.render_stats.binds += 2;
                    last_pipeline = pipeline.id();
                }

                if let Some(material_binding) = &render_object.mesh.material_binding {
                    log::trace!("Bind material");
                    cmd.bind_resource(material_binding);
                    batch.render_stats.binds += 1;
                }

                last_material = material.id;
            }
            batch.render_stats.cpu_draw_bind += timer.delta();

            log::trace!("Bind push constants");
            if let Some(push_layout) = render_object.pass.push_layout() {
                model_data.clear();
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
                    &mut model_data,
                );

                cmd.push_constants(&pipeline, &model_data);
            }
            batch.render_stats.cpu_draw_push += timer.delta();

            if last_buffer != render_object.mesh.index_buffer.id()
                || last_offset != render_object.mesh.indices_offset
            {
                cmd.bind_index_buffer(
                    &render_object.mesh.index_buffer,
                    render_object.mesh.indices_offset,
                );
                batch.render_stats.binds += 1;
                last_buffer = render_object.mesh.index_buffer.id();
                last_offset = render_object.mesh.indices_offset;
            }
            batch.render_stats.cpu_draw_bind += timer.delta();

            cmd.draw_indexed(render_object.mesh.indices_len, 1);
            batch.render_stats.draws += 1;
            batch.render_stats.cpu_draw_submit += timer.delta();
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
