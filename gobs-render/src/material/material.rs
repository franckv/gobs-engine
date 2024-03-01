use std::{path::PathBuf, sync::Arc};

use parking_lot::RwLock;
use uuid::Uuid;

use gobs_utils::load;
use gobs_vulkan::{
    descriptor::{
        DescriptorSetLayout, DescriptorSetLayoutBuilder, DescriptorSetPool, DescriptorStage,
        DescriptorType,
    },
    image::ImageLayout,
    pipeline::{
        BlendMode, CompareOp, CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout,
        Rect2D, Shader, ShaderType, Viewport,
    },
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    material::{MaterialInstance, Texture},
    pass::RenderPass,
};

pub type MaterialId = Uuid;

pub struct Material {
    pub id: MaterialId,
    pub vertex_flags: VertexFlag,
    pub pipeline: Arc<Pipeline>,
    pub blending_enabled: bool,
    pub material_ds_pool: Option<RwLock<DescriptorSetPool>>,
}

impl Material {
    pub fn builder(vertex_shader: &str, fragment_shader: &str) -> MaterialBuilder {
        MaterialBuilder::new(vertex_shader, fragment_shader)
    }

    pub fn instantiate(self: &Arc<Self>, textures: Vec<Texture>) -> Arc<MaterialInstance> {
        let material_ds = match &self.material_ds_pool {
            Some(ds_pool) => {
                let material_ds = ds_pool.write().allocate();
                let mut updater = material_ds.update();

                for texture in &textures {
                    updater = updater
                        .bind_sampled_image(&texture.read().image, ImageLayout::Shader)
                        .bind_sampler(&texture.read().sampler);
                }

                updater.end();

                Some(material_ds)
            }
            None => None,
        };

        MaterialInstance::new(self.clone(), material_ds, textures)
    }
}

pub enum MaterialProperty {
    Texture,
}

pub struct MaterialBuilder {
    pub vertex_shader: PathBuf,
    pub fragment_shader: PathBuf,
    pub vertex_flags: VertexFlag,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    pub material_descriptor_layout: Option<DescriptorSetLayoutBuilder>,
}

impl MaterialBuilder {
    pub fn new(vertex_shader: &str, fragment_shader: &str) -> Self {
        let vertex_shader = load::get_asset_dir(vertex_shader, load::AssetType::SHADER).unwrap();
        let fragment_shader =
            load::get_asset_dir(fragment_shader, load::AssetType::SHADER).unwrap();

        Self {
            vertex_shader,
            fragment_shader,
            vertex_flags: VertexFlag::empty(),
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
            material_descriptor_layout: None,
        }
    }

    pub fn vertex_flags(mut self, vertex_flags: VertexFlag) -> Self {
        self.vertex_flags = vertex_flags;

        self
    }

    pub fn no_culling(mut self) -> Self {
        self.cull_mode = CullMode::None;

        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;

        self
    }

    pub fn prop(mut self, _name: &str, prop: MaterialProperty) -> Self {
        let mut builder = match self.material_descriptor_layout {
            Some(builder) => builder,
            None => DescriptorSetLayout::builder(),
        };

        match prop {
            MaterialProperty::Texture => {
                builder = builder.binding(DescriptorType::SampledImage, DescriptorStage::Fragment);
                builder = builder.binding(DescriptorType::Sampler, DescriptorStage::Fragment);
            }
        }

        self.material_descriptor_layout = Some(builder);

        self
    }

    pub fn build(self, ctx: &Context, pass: Arc<dyn RenderPass>) -> Arc<Material> {
        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let material_descriptor_layout = self
            .material_descriptor_layout
            .map(|builder| builder.build(ctx.device.clone()));

        let mut ds_layouts = vec![scene_descriptor_layout.clone()];
        if let Some(ref material_layout) = material_descriptor_layout {
            ds_layouts.push(material_layout.clone());
        }

        let pipeline_layout = PipelineLayout::new(
            ctx.device.clone(),
            &ds_layouts,
            match pass.push_layout() {
                Some(push_layout) => push_layout.size(),
                None => 0,
            },
        );

        let vertex_shader =
            Shader::from_file(self.vertex_shader, ctx.device.clone(), ShaderType::Vertex);
        let fragment_shader = Shader::from_file(
            self.fragment_shader,
            ctx.device.clone(),
            ShaderType::Fragment,
        );

        let pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .vertex_shader("main", vertex_shader)
            .fragment_shader("main", fragment_shader)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(Some(ctx.color_format), Some(ctx.depth_format))
            .depth_test_enable(false, CompareOp::LessEqual)
            .cull_mode(self.cull_mode)
            .blending_enabled(self.blend_mode)
            .front_face(FrontFace::CCW)
            .build();

        let material_ds_pool = material_descriptor_layout
            .map(|ds_layout| RwLock::new(DescriptorSetPool::new(ctx.device.clone(), ds_layout, 1)));

        Arc::new(Material {
            id: Uuid::new_v4(),
            vertex_flags: self.vertex_flags,
            pipeline,
            blending_enabled: self.blend_mode != BlendMode::None,
            material_ds_pool,
        })
    }
}
