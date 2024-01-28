use std::sync::Arc;

use glam::Vec3;
use gobs_core::entity::{
    camera::Camera,
    uniform::{UniformLayout, UniformProp},
};
use gobs_render::context::Context;
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorStage, DescriptorType},
    image::{ImageExtent2D, ImageFormat},
    pipeline::{
        CompareOp, CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout, Rect2D, Shader,
        ShaderType, Viewport,
    },
};

use crate::graph::scenegraph::SceneGraph;

const SHADER_DIR: &str = "examples/shaders";

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub pipeline: Pipeline,
    pub pipeline_layout: Arc<PipelineLayout>,
    pub scene_descriptor_layout: Arc<DescriptorSetLayout>,
    pub scene_data_layout: Arc<UniformLayout>,
    pub model_data_layout: Arc<UniformLayout>,
}

impl Scene {
    pub fn new(
        ctx: &Context,
        size: ImageExtent2D,
        color_format: ImageFormat,
        depth_format: Option<ImageFormat>,
    ) -> Self {
        let vertex_shader = Shader::from_file(
            &format!("{}/mesh.vert.spv", SHADER_DIR),
            ctx.device.clone(),
            ShaderType::Vertex,
        );

        let fragment_shader = Shader::from_file(
            &format!("{}/mesh.frag.spv", SHADER_DIR),
            ctx.device.clone(),
            ShaderType::Fragment,
        );

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let scene_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
            .build();

        let model_data_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let pipeline_layout = PipelineLayout::with_constants(
            ctx.device.clone(),
            Some(scene_descriptor_layout.clone()),
            model_data_layout.size(),
        );
        let pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .vertex_shader("main", vertex_shader)
            .fragment_shader("main", fragment_shader)
            .viewports(vec![Viewport::new(
                0.,
                0.,
                size.width as f32,
                size.height as f32,
            )])
            .scissors(vec![Rect2D::new(0, 0, size.width, size.height)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(color_format, depth_format)
            .depth_test_enable(true, CompareOp::Less)
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

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

        Scene {
            graph: SceneGraph::new(),
            camera,
            pipeline,
            pipeline_layout,
            scene_descriptor_layout,
            scene_data_layout,
            model_data_layout,
        }
    }
}
