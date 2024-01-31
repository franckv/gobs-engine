use std::sync::Arc;

use uuid::Uuid;

use gobs_core::{entity::uniform::UniformLayout, Color};
use gobs_render::context::Context;
use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    image::{ImageLayout, SamplerFilter},
    pipeline::{
        CompareOp, CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout, Rect2D, Shader,
        ShaderType, Viewport,
    },
};

use crate::texture::Texture;

const SHADER_DIR: &str = "examples/shaders";

pub type MaterialId = Uuid;

pub struct Material {
    pub id: MaterialId,
    pub pipeline: Arc<Pipeline>,
    pub texture: Texture,
    pub material_ds_pool: DescriptorSetPool,
    pub material_ds: DescriptorSet,
}

impl Material {
    pub fn new(ctx: &Context, model_data_layout: Arc<UniformLayout>) -> Self {
        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::Vertex)
            .build(ctx.device.clone());

        let material_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::SampledImage, DescriptorStage::Fragment)
            .binding(DescriptorType::Sampler, DescriptorStage::Fragment)
            .build(ctx.device.clone());

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

        let pipeline_layout = PipelineLayout::new(
            ctx.device.clone(),
            &[
                scene_descriptor_layout.clone(),
                material_descriptor_layout.clone(),
            ],
            model_data_layout.size(),
        );
        let pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .vertex_shader("main", vertex_shader)
            .fragment_shader("main", fragment_shader)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(ctx.color_format, Some(ctx.depth_format))
            .depth_test_enable(true, CompareOp::Less)
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        let texture = Texture::with_color(
            ctx,
            Color::from_rgba8(200, 200, 150, 255),
            SamplerFilter::FilterLinear,
        );

        let mut material_ds_pool =
            DescriptorSetPool::new(ctx.device.clone(), material_descriptor_layout, 1);

        let material_ds = material_ds_pool.allocate();

        material_ds
            .update()
            .bind_sampled_image(&texture.image, ImageLayout::Shader)
            .bind_sampler(&texture.sampler)
            .end();

        Material {
            id: Uuid::new_v4(),
            pipeline,
            texture,
            material_ds_pool,
            material_ds,
        }
    }
}
