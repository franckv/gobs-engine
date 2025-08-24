use gobs_gfx::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType, FrontFace,
};
use gobs_render_graph::{GraphicsPipelineProperties, Pipeline, PipelineProperties};
use gobs_render_low::{
    GfxContext, MaterialDataLayout, MaterialDataProp, ObjectDataLayout, TextureDataLayout,
    TextureDataProp, UniformData,
};
use gobs_resource::{
    geometry::VertexAttribute,
    resource::{ResourceHandle, ResourceProperties, ResourceType},
};

use crate::resources::MaterialLoader;

#[derive(Clone, Copy, Debug)]
pub struct Material;

impl ResourceType for Material {
    type ResourceData = MaterialData;
    type ResourceProperties = MaterialProperties;
    type ResourceParameter = ();
    type ResourceLoader = MaterialLoader;
}

#[derive(Clone, Debug)]
pub struct MaterialProperties {
    pub name: String,
    pub pipeline_properties: GraphicsPipelineProperties,
    pub blending_enabled: bool,
    pub texture_data_layout: TextureDataLayout,
    pub material_data_layout: MaterialDataLayout,
}

impl ResourceProperties for MaterialProperties {
    fn name(&self) -> &str {
        &self.name
    }
}

impl MaterialProperties {
    #[allow(clippy::too_many_arguments)]
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
        let pipeline_properties = PipelineProperties::graphics(name)
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
            texture_data_layout: TextureDataLayout::default(),
            material_data_layout: MaterialDataLayout::default(),
        }
    }

    pub fn property(mut self, prop: MaterialDataProp) -> Self {
        if self.pipeline_properties.last_binding_group != BindingGroupType::MaterialData {
            self.pipeline_properties = self
                .pipeline_properties
                .binding_group(DescriptorStage::Fragment, BindingGroupType::MaterialData);
            self.pipeline_properties = self.pipeline_properties.binding(DescriptorType::Uniform);
        }

        self.material_data_layout = self.material_data_layout.prop(prop);

        self
    }

    pub fn texture(mut self, prop: TextureDataProp) -> Self {
        if self.pipeline_properties.last_binding_group != BindingGroupType::MaterialTextures {
            self.pipeline_properties = self.pipeline_properties.binding_group(
                DescriptorStage::Fragment,
                BindingGroupType::MaterialTextures,
            );
        }

        self.texture_data_layout = self.texture_data_layout.prop(prop);

        match prop {
            TextureDataProp::Diffuse => {
                self.pipeline_properties = self
                    .pipeline_properties
                    .binding(DescriptorType::SampledImage)
                    .binding(DescriptorType::Sampler);
            }
            TextureDataProp::Normal => {
                self.pipeline_properties = self
                    .pipeline_properties
                    .binding(DescriptorType::SampledImage)
                    .binding(DescriptorType::Sampler);
            }
            _ => unimplemented!(),
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
