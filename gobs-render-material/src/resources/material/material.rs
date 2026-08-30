use gobs_core::ImageFormat;
use gobs_graphics::VertexAttribute;
use gobs_render_hal::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType, FrontFace,
    ObjectDataLayout, UniformData as _,
};
use gobs_resource::{ResourceHandle, ResourceProperties, ResourceType};

use crate::{
    data::{
        MaterialDataLayout, MaterialDataProp, SceneDataLayout, TextureDataLayout, TextureDataProp,
    },
    resources::{GraphicsPipelineProperties, Pipeline, PipelineProperties},
};

#[derive(Clone, Copy, Debug)]
pub struct Material;

impl ResourceType for Material {
    type ResourceData = MaterialData;
    type ResourceProperties = MaterialProperties;
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
        name: &str,
        vertex_shader: &str,
        vertex_entry: &str,
        fragment_shader: &str,
        fragment_entry: &str,
        vertex_attributes: VertexAttribute,
        object_data_layout: ObjectDataLayout,
        push_data_layout: ObjectDataLayout,
        scene_data_layout: SceneDataLayout,
        color_format: ImageFormat,
        depth_format: ImageFormat,
        texture_indexing: bool,
        material_indexing: bool,
    ) -> Self {
        let pipeline_properties = PipelineProperties::graphics(name)
            .vertex_shader(vertex_shader)
            .vertex_entry(vertex_entry)
            .fragment_shader(fragment_shader)
            .fragment_entry(fragment_entry)
            .pool_size(10)
            .object_data_layout(object_data_layout)
            .push_data_layout(push_data_layout)
            .scene_data_layout(scene_data_layout)
            .vertex_attributes(vertex_attributes)
            .depth_test_enable(false, CompareOp::LessEqual)
            .front_face(FrontFace::CCW)
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::All, 1)
            .color_format(color_format)
            .depth_format(depth_format);

        let texture_data_layout = TextureDataLayout::new(texture_indexing);

        Self {
            name: name.to_string(),
            pipeline_properties,
            blending_enabled: false,
            texture_data_layout,
            material_data_layout: MaterialDataLayout::new(material_indexing),
        }
    }

    pub fn property(mut self, prop: MaterialDataProp) -> Self {
        let material_indexing = self.material_data_layout.material_indexing;

        let binding_group_type = if material_indexing {
            BindingGroupType::BindlessMaterial
        } else {
            BindingGroupType::MaterialData
        };

        let descriptor_type = if material_indexing {
            DescriptorType::StorageBuffer
        } else {
            DescriptorType::Uniform
        };

        if self
            .pipeline_properties
            .binding_groups
            .last()
            .is_none_or(|group| group.binding_group_type != binding_group_type)
        {
            self.pipeline_properties = self
                .pipeline_properties
                .binding_group(binding_group_type)
                .binding(descriptor_type, DescriptorStage::Fragment, 1);
        }

        self.material_data_layout = self.material_data_layout.prop(prop);

        self
    }

    pub fn textures(mut self, props: &[TextureDataProp], array_size: u32) -> Self {
        if props.is_empty() {
            return self;
        }

        let texture_indexing = self.texture_data_layout.texture_indexing;

        debug_assert!(!texture_indexing || array_size > 0);

        let binding_group_type = if texture_indexing {
            BindingGroupType::BindlessTextures
        } else {
            BindingGroupType::MaterialTextures
        };

        if self
            .pipeline_properties
            .binding_groups
            .last()
            .is_none_or(|group| group.binding_group_type != binding_group_type)
        {
            self.pipeline_properties = self.pipeline_properties.binding_group(binding_group_type);
        }

        if texture_indexing {
            self.pipeline_properties = self
                .pipeline_properties
                .binding(DescriptorType::Sampler, DescriptorStage::Fragment, 1)
                .binding(
                    DescriptorType::SampledImage,
                    DescriptorStage::Fragment,
                    array_size,
                );
        } else {
            for &prop in props {
                match prop {
                    TextureDataProp::Diffuse => {
                        self.pipeline_properties = self
                            .pipeline_properties
                            .binding(DescriptorType::SampledImage, DescriptorStage::Fragment, 1)
                            .binding(DescriptorType::Sampler, DescriptorStage::Fragment, 1);
                    }
                    TextureDataProp::Normal => {
                        self.pipeline_properties = self
                            .pipeline_properties
                            .binding(DescriptorType::SampledImage, DescriptorStage::Fragment, 1)
                            .binding(DescriptorType::Sampler, DescriptorStage::Fragment, 1);
                    }
                    _ => unimplemented!(),
                }
            }
        }

        for &prop in props {
            self.texture_data_layout = self.texture_data_layout.prop(prop);
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

    pub fn depth_test_disable(mut self) -> Self {
        self.pipeline_properties = self.pipeline_properties.depth_test_disable();

        self
    }
}

#[derive(Clone)]
pub struct MaterialData {
    pub pipeline: ResourceHandle<Pipeline>,
}
