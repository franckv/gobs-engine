use std::sync::Arc;

use uuid::Uuid;

use gobs_gfx::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType, FrontFace,
};
use gobs_render_graph::{GfxContext, RenderError, RenderPass};
use gobs_resource::{
    geometry::VertexAttribute,
    manager::ResourceManager,
    resource::{ResourceHandle, ResourceLifetime},
};

use crate::{
    Pipeline, PipelineProperties, Texture, materials::MaterialInstance,
    resources::GraphicsPipelineProperties,
};

pub type MaterialId = Uuid;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub id: MaterialId,
    pub vertex_attributes: VertexAttribute,
    pub pipeline: ResourceHandle<Pipeline>,
    pub blending_enabled: bool,
}

impl Material {
    pub fn builder(
        ctx: &GfxContext,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<MaterialBuilder, RenderError> {
        MaterialBuilder::new(ctx, vertex_shader, fragment_shader)
    }

    pub fn instantiate(
        self: &Arc<Self>,
        textures: Vec<ResourceHandle<Texture>>,
    ) -> Arc<MaterialInstance> {
        MaterialInstance::new(self.clone(), textures)
    }
}

pub enum MaterialProperty {
    Texture,
}

pub struct MaterialBuilder {
    vertex_attributes: VertexAttribute,
    blend_mode: BlendMode,
    pipeline_properties: GraphicsPipelineProperties,
    last_binding_group_type: Option<BindingGroupType>,
}

impl MaterialBuilder {
    pub fn new(
        ctx: &GfxContext,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Self, RenderError> {
        let pipeline_properties = PipelineProperties::graphics("material")
            .vertex_shader(vertex_shader)
            .fragment_shader(fragment_shader)
            .pool_size(ctx.frames_in_flight + 1)
            .color_format(ctx.color_format)
            .depth_format(ctx.depth_format)
            .depth_test_enable(false, CompareOp::LessEqual)
            .front_face(FrontFace::CCW)
            .binding_group(DescriptorStage::All, BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform);

        Ok(Self {
            vertex_attributes: VertexAttribute::empty(),
            blend_mode: BlendMode::None,
            pipeline_properties,
            last_binding_group_type: None,
        })
    }

    pub fn vertex_attributes(mut self, vertex_attributes: VertexAttribute) -> Self {
        self.vertex_attributes = vertex_attributes;

        self
    }

    pub fn no_culling(mut self) -> Self {
        self.pipeline_properties = self.pipeline_properties.cull_mode(CullMode::None);

        self
    }

    pub fn cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.pipeline_properties = self.pipeline_properties.cull_mode(cull_mode);

        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.pipeline_properties = self.pipeline_properties.blend_mode(blend_mode);
        self.blend_mode = blend_mode;

        self
    }

    pub fn prop(mut self, _name: &str, prop: MaterialProperty) -> Self {
        if self.last_binding_group_type != Some(BindingGroupType::MaterialData) {
            self.pipeline_properties = self
                .pipeline_properties
                .binding_group(DescriptorStage::Fragment, BindingGroupType::MaterialData);
            self.last_binding_group_type = Some(BindingGroupType::MaterialData);
        }

        match prop {
            MaterialProperty::Texture => {
                self.pipeline_properties = self
                    .pipeline_properties
                    .binding(DescriptorType::SampledImage)
                    .binding(DescriptorType::Sampler);
            }
        }

        self
    }

    pub fn build(self, pass: RenderPass, resource_manager: &mut ResourceManager) -> Arc<Material> {
        let pipeline_properties = match pass.push_layout() {
            Some(push_layout) => self.pipeline_properties.push_constants(push_layout.size()),
            None => self.pipeline_properties,
        };

        let pipeline = resource_manager.add(pipeline_properties.wrap(), ResourceLifetime::Static);

        Arc::new(Material {
            id: Uuid::new_v4(),
            vertex_attributes: self.vertex_attributes,
            pipeline,
            blending_enabled: self.blend_mode != BlendMode::None,
        })
    }
}
