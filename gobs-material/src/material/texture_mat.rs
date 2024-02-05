use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core::entity::uniform::{UniformLayout, UniformProp};
use gobs_render::context::Context;
use gobs_utils::load;
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType},
    image::ImageLayout,
    pipeline::{
        CompareOp, CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout, Rect2D, Shader,
        ShaderType, Viewport,
    },
};

use crate::{
    instance::MaterialInstance, texture::Texture, vertex::VertexFlag, Material, MaterialId,
};

pub struct TextureMaterial {
    pub id: MaterialId,
    pub vertex_flags: VertexFlag,
    pub pipeline: Arc<Pipeline>,

    pub material_ds_pool: RwLock<DescriptorSetPool>,
    pub model_data_layout: Arc<UniformLayout>,
}

impl TextureMaterial {
    pub fn new(ctx: &Context) -> Arc<Material> {
        let model_data_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let material_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::SampledImage, DescriptorStage::Fragment)
            .binding(DescriptorType::Sampler, DescriptorStage::Fragment)
            .build(ctx.device.clone());

        let vertex_flags =
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE | VertexFlag::NORMAL;

        let vertex_file = load::get_asset_dir("mesh.vert.spv", load::AssetType::SHADER).unwrap();
        let vertex_shader = Shader::from_file(vertex_file, ctx.device.clone(), ShaderType::Vertex);

        let fragment_file = load::get_asset_dir("mesh.frag.spv", load::AssetType::SHADER).unwrap();
        let fragment_shader =
            Shader::from_file(fragment_file, ctx.device.clone(), ShaderType::Fragment);

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

        let material_ds_pool =
            DescriptorSetPool::new(ctx.device.clone(), material_descriptor_layout, 1);

        Arc::new(Material::Texture(TextureMaterial {
            id: Uuid::new_v4(),
            vertex_flags,
            pipeline,
            material_ds_pool: RwLock::new(material_ds_pool),
            model_data_layout,
        }))
    }

    pub fn instanciate(material: Arc<Material>, texture: Texture) -> Arc<MaterialInstance> {
        let material_ds = material.ds_pool().write().unwrap().allocate();

        material_ds
            .update()
            .bind_sampled_image(&texture.image, ImageLayout::Shader)
            .bind_sampler(&texture.sampler)
            .end();

        MaterialInstance::new(material.clone(), material_ds, texture)
    }
}

impl Drop for TextureMaterial {
    fn drop(&mut self) {
        log::debug!("Drop material");
    }
}
