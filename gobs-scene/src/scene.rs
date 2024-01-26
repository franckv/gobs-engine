use std::sync::Arc;

use glam::{Mat4, Vec3};
use gobs_core::entity::{
    camera::Camera,
    uniform::{UniformData, UniformPropData},
};
use gobs_render::context::Context;
use gobs_vulkan::{
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

        let scene_data = UniformData::builder("scene data")
            .prop(
                "world_matrix",
                UniformPropData::Mat4F(Mat4::IDENTITY.to_cols_array_2d()),
            )
            .prop("vertex_buffer", UniformPropData::U64(0))
            .build();

        let pipeline_layout =
            PipelineLayout::with_constants(ctx.device.clone(), None, scene_data.raw().len());
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
            (70. as f32).to_radians(),
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
        }
    }
}
