use std::sync::Arc;

use glam::Vec3;
use gobs_core::entity::{
    camera::Camera,
    uniform::{UniformData, UniformLayout, UniformProp, UniformPropData},
};

use gobs_render::context::Context;
use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    image::ImageExtent2D,
};

use crate::{graph::scenegraph::SceneGraph, uniform_buffer::UniformBuffer};

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

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::Vertex)
            .build(ctx.device.clone());

        let scene_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
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
            ],
        );

        self.scene_frame_data[frame_number % ctx.frames_in_flight]
            .uniform_buffer
            .update(&scene_data);
    }

    pub fn uniform_ds(&self, ctx: &Context, frame_number: usize) -> &DescriptorSet {
        &self.scene_frame_data[frame_number % ctx.frames_in_flight].uniform_ds
    }
}
