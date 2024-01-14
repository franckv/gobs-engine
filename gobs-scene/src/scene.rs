use std::sync::Arc;

use glam::Mat4;
use gobs_core::entity::uniform::{UniformData, UniformProp};
use gobs_render::context::Context;
use gobs_vulkan::{
    image::{ImageExtent2D, ImageFormat},
    pipeline::{
        DynamicStateElem, FrontFace, Pipeline, PipelineLayout, Rect2D, Shader, ShaderType, Viewport,
    },
};

use crate::model::Model;

const SHADER_DIR: &str = "examples/shaders";

pub struct Scene {
    pub models: Vec<Model>,
    pub scene_data: UniformData,
    pub pipeline: Pipeline,
    pub pipeline_layout: Arc<PipelineLayout>,
}

impl Scene {
    pub fn new(ctx: &Context, size: ImageExtent2D, format: ImageFormat) -> Self {
        let vertex_shader = Shader::from_file(
            &format!("{}/mesh.vert.spv", SHADER_DIR),
            ctx.device.clone(),
            ShaderType::Vertex,
        );

        let fragment_shader = Shader::from_file(
            &format!("{}/triangle.frag.spv", SHADER_DIR),
            ctx.device.clone(),
            ShaderType::Fragment,
        );

        let scene_data = UniformData::builder("scene data")
            .prop(
                "world_matrix",
                UniformProp::Mat4F(Mat4::from_scale([0.5, 0.5, 1.].into()).to_cols_array_2d()),
            )
            .prop("vertex_buffer", UniformProp::U64(0))
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
            .attachments(format, None)
            .depth_test_disable()
            .front_face(FrontFace::CW)
            .build();

        Scene {
            models: Vec::new(),
            scene_data,
            pipeline,
            pipeline_layout,
        }
    }
}
