use serde::Deserialize;

use gobs_gfx::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType, FrontFace,
};
use gobs_render_graph::{GraphicsPipelineProperties, Pipeline, PipelineProperties};
use gobs_render_low::{GfxContext, ObjectDataLayout};
use gobs_resource::{
    geometry::VertexAttribute,
    resource::{ResourceHandle, ResourceProperties, ResourceType},
};

use crate::resources::material_loader::MaterialLoader;

#[derive(Clone, Copy, Debug)]
pub struct Material;

impl ResourceType for Material {
    type ResourceData = MaterialData;
    type ResourceProperties = MaterialProperties;
    type ResourceParameter = ();
    type ResourceLoader = MaterialLoader;
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum MaterialProperty {
    Texture,
}

#[derive(Clone, Debug)]
pub struct MaterialProperties {
    pub name: String,
    pub pipeline_properties: GraphicsPipelineProperties,
    pub blending_enabled: bool,
}

impl ResourceProperties for MaterialProperties {
    fn name(&self) -> &str {
        &self.name
    }
}

impl MaterialProperties {
    pub fn new(
        ctx: &GfxContext,
        name: &str,
        vertex_shader: &str,
        vertex_entry: &str,
        fragment_shader: &str,
        fragment_entry: &str,
        vertex_attributes: VertexAttribute,
        object_data_layout: &ObjectDataLayout,
    ) -> Self {
        let pipeline_properties = PipelineProperties::graphics("material")
            .vertex_shader(vertex_shader)
            .vertex_entry(vertex_entry)
            .fragment_shader(fragment_shader)
            .fragment_entry(fragment_entry)
            .pool_size(10)
            .push_constants(object_data_layout.uniform_layout().size())
            .vertex_attributes(vertex_attributes)
            .depth_test_enable(false, CompareOp::LessEqual)
            .front_face(FrontFace::CCW)
            .binding_group(DescriptorStage::All, BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform)
            .color_format(ctx.color_format)
            .depth_format(ctx.depth_format);

        Self {
            name: name.to_string(),
            pipeline_properties,
            blending_enabled: false,
        }
    }

    pub fn prop(mut self, _name: &str, prop: MaterialProperty) -> Self {
        if self.pipeline_properties.last_binding_group != BindingGroupType::MaterialTextures {
            self.pipeline_properties = self.pipeline_properties.binding_group(
                DescriptorStage::Fragment,
                BindingGroupType::MaterialTextures,
            );
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
        self.blending_enabled = blend_mode != BlendMode::None;

        self
    }
}

#[derive(Clone)]
pub struct MaterialData {
    pub pipeline: ResourceHandle<Pipeline>,
}
