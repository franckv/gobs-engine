use std::sync::Arc;

use glam::Vec3;
use gobs_core::entity::{
    camera::Camera,
    uniform::{UniformLayout, UniformProp},
};

use gobs_render::context::Context;
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorStage, DescriptorType},
    image::ImageExtent2D,
};

use crate::graph::scenegraph::SceneGraph;

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub scene_descriptor_layout: Arc<DescriptorSetLayout>,
    pub scene_data_layout: Arc<UniformLayout>,
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

        Scene {
            graph: SceneGraph::new(),
            camera,
            scene_descriptor_layout,
            scene_data_layout,
        }
    }
}
