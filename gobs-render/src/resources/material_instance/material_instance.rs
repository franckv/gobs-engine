use std::sync::Arc;

use gobs_gfx::{GfxBindingGroup, GfxBuffer};
use gobs_render_low::{
    MaterialConstantData, MaterialDataLayout, MaterialDataProp, MaterialDataPropData, UniformData,
};
use gobs_resource::resource::{ResourceHandle, ResourceProperties, ResourceType};

use crate::resources::{Material, MaterialInstanceLoader, Texture};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialInstance;

impl ResourceType for MaterialInstance {
    type ResourceData = MaterialInstanceData;
    type ResourceProperties = MaterialInstanceProperties;
    type ResourceParameter = ();
    type ResourceLoader = MaterialInstanceLoader;
}

#[derive(Clone, Debug)]
pub struct MaterialInstanceProperties {
    pub name: String,
    pub material_data: Option<MaterialConstantData>,
    pub material_data_layout: MaterialDataLayout,
    pub material: ResourceHandle<Material>,
    pub textures: Vec<ResourceHandle<Texture>>,
}

impl ResourceProperties for MaterialInstanceProperties {
    fn name(&self) -> &str {
        &self.name
    }
}

impl MaterialInstanceProperties {
    pub fn new(name: &str, material: ResourceHandle<Material>) -> Self {
        Self {
            name: name.to_string(),
            material_data: None,
            material_data_layout: Default::default(),
            material,
            textures: Vec::new(),
        }
    }

    pub fn textures(mut self, textures: &[ResourceHandle<Texture>]) -> Self {
        self.textures.extend_from_slice(textures);

        self
    }

    pub fn prop(mut self, prop: MaterialDataPropData) -> Self {
        let mut material_data = self.material_data.unwrap_or_default();

        match prop {
            MaterialDataPropData::DiffuseColor(color) => {
                material_data.diffuse_color = color;
                self.material_data_layout = self
                    .material_data_layout
                    .prop(MaterialDataProp::DiffuseColor);
            }
            MaterialDataPropData::EmissionColor(color) => {
                material_data.emission_color = color;
                self.material_data_layout = self
                    .material_data_layout
                    .prop(MaterialDataProp::EmissionColor);
            }
            MaterialDataPropData::SpecularColor(color) => {
                material_data.specular_color = color;
                self.material_data_layout = self
                    .material_data_layout
                    .prop(MaterialDataProp::SpecularColor);
            }
            MaterialDataPropData::SpecularPower(power) => {
                material_data.specular_power = power;
                self.material_data_layout = self
                    .material_data_layout
                    .prop(MaterialDataProp::SpecularPower);
            }
        }

        self.material_data = Some(material_data);

        self
    }
}

pub struct MaterialInstanceData {
    pub material: ResourceHandle<Material>,
    pub material_buffer: Option<Arc<GfxBuffer>>,
    pub material_binding: Option<GfxBindingGroup>,
    pub texture_binding: Option<GfxBindingGroup>,
    pub bound: bool,
}
