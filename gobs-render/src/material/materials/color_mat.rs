use std::sync::Arc;

use uuid::Uuid;

use gobs_core::entity::uniform::{UniformLayout, UniformProp};
use gobs_utils::load;
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorStage, DescriptorType},
    pipeline::{
        CompareOp, CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout, Rect2D, Shader,
        ShaderType, Viewport,
    },
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    material::{Material, MaterialId, MaterialInstance},
};

pub struct ColorMaterial {
    pub id: MaterialId,
    pub vertex_flags: VertexFlag,
    pub pipeline: Arc<Pipeline>,
    pub model_data_layout: Arc<UniformLayout>,
}

impl ColorMaterial {
    pub fn new(ctx: &Context) -> Arc<Material> {
        let model_data_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("normal_matrix", UniformProp::Mat3F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        let vertex_file = load::get_asset_dir("color.vert.spv", load::AssetType::SHADER).unwrap();
        let vertex_shader = Shader::from_file(vertex_file, ctx.device.clone(), ShaderType::Vertex);

        let fragment_file = load::get_asset_dir("color.frag.spv", load::AssetType::SHADER).unwrap();
        let fragment_shader =
            Shader::from_file(fragment_file, ctx.device.clone(), ShaderType::Fragment);

        let pipeline_layout = PipelineLayout::new(
            ctx.device.clone(),
            &[scene_descriptor_layout.clone()],
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

        Arc::new(Material::Color(ColorMaterial {
            id: Uuid::new_v4(),
            vertex_flags,
            pipeline,
            model_data_layout,
        }))
    }

    pub fn instanciate(material: Arc<Material>) -> Arc<MaterialInstance> {
        MaterialInstance::new(material.clone(), None, vec![])
    }
}

impl Drop for ColorMaterial {
    fn drop(&mut self) {
        log::debug!("Drop material");
    }
}
