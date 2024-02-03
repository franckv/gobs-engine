use std::sync::Arc;

use glam::Vec3;
use gobs_core::{
    entity::{
        camera::Camera,
        light::Light,
        uniform::{UniformData, UniformLayout, UniformProp, UniformPropData},
    },
    Color,
};

use gobs_render::{context::Context, CommandBuffer};
use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    image::ImageExtent2D,
};
use uuid::Uuid;

use crate::{
    graph::scenegraph::{NodeValue, SceneGraph},
    uniform_buffer::UniformBuffer,
};

struct SceneFrameData {
    pub uniform_ds: DescriptorSet,
    pub uniform_buffer: UniformBuffer,
}

impl SceneFrameData {
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

        SceneFrameData {
            uniform_ds,
            uniform_buffer,
        }
    }
}

#[allow(unused)]
pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub light: Light,
    pub scene_descriptor_layout: Arc<DescriptorSetLayout>,
    pub scene_data_layout: Arc<UniformLayout>,
    scene_ds_pool: DescriptorSetPool,
    scene_frame_data: Vec<SceneFrameData>,
}

impl Scene {
    pub fn new(ctx: &Context, size: ImageExtent2D) -> Self {
        let camera = Camera::perspective(
            Vec3::splat(0.),
            size.width as f32 / size.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            (-90. as f32).to_radians(),
            0.,
            Vec3::Y,
        );

        let light = Light::new(Vec3::new(-3., 0., 5.), Color::WHITE);

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let scene_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
            .prop("light_direction", UniformProp::Vec3F)
            .prop("light_color", UniformProp::Vec4F)
            .prop("ambient_color", UniformProp::Vec4F)
            .build();

        let mut scene_ds_pool = DescriptorSetPool::new(
            ctx.device.clone(),
            scene_descriptor_layout.clone(),
            ctx.frames_in_flight as u32,
        );

        let scene_frame_data = (0..ctx.frames_in_flight)
            .map(|_| SceneFrameData::new(ctx, scene_data_layout.clone(), scene_ds_pool.allocate()))
            .collect();

        Scene {
            graph: SceneGraph::new(),
            camera,
            light,
            scene_descriptor_layout,
            scene_data_layout,
            scene_ds_pool,
            scene_frame_data,
        }
    }

    pub fn update(&mut self, ctx: &Context, frame_number: usize) {
        let scene_data = UniformData::new(
            &self.scene_data_layout,
            &[
                UniformPropData::Vec3F(self.camera.position.into()),
                UniformPropData::Mat4F(self.camera.view_proj().to_cols_array_2d()),
                UniformPropData::Vec3F(self.light.position.into()),
                UniformPropData::Vec4F(self.light.colour.into()),
                UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]),
            ],
        );

        self.scene_frame_data[frame_number % ctx.frames_in_flight]
            .uniform_buffer
            .update(&scene_data);
    }

    pub fn draw(&self, ctx: &Context, cmd: &CommandBuffer, frame_number: usize) {
        let mut last_material = Uuid::nil();
        self.graph.visit(self.graph.root, &mut |transform, model| {
            if let NodeValue::Model(model) = model {
                let world_matrix = transform.matrix;

                let model_data = UniformData::new(
                    &model.model_data_layout,
                    &[
                        UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        UniformPropData::U64(model.vertex_buffer.address(ctx.device.clone())),
                    ],
                );

                for primitive in &model.primitives {
                    let material = &model.materials[primitive.material];
                    let pipeline = &material.pipeline;

                    if last_material != material.id {
                        cmd.bind_pipeline(&material.pipeline);
                        cmd.bind_descriptor_set(&self.uniform_ds(ctx, frame_number), 0, pipeline);
                        cmd.bind_descriptor_set(&material.material_ds, 1, pipeline);
                        last_material = material.id;
                    }

                    cmd.push_constants(material.pipeline.layout.clone(), &model_data.raw());
                    cmd.bind_index_buffer::<u32>(&model.index_buffer, primitive.offset);
                    cmd.draw_indexed(primitive.len, 1);
                }
            }
        });
    }

    pub fn uniform_ds(&self, ctx: &Context, frame_number: usize) -> &DescriptorSet {
        &self.scene_frame_data[frame_number % ctx.frames_in_flight].uniform_ds
    }
}
