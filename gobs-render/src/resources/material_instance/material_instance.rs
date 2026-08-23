use gobs_render_hal::RenderHAL;
use gobs_resource::{ResourceHandle, ResourceProperties, ResourceType};

use crate::{
    data::{MaterialConstantData, MaterialDataPropData},
    material_system::MaterialBinding,
    resources::{Material, MaterialInstanceLoader, Texture},
};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct MaterialInstance;

impl ResourceType for MaterialInstance {
    type ResourceData = MaterialInstanceData;
    type ResourceBackend<'a> = dyn RenderHAL + 'a;
    type ResourceProperties = MaterialInstanceProperties;
    type ResourceLoader = MaterialInstanceLoader;
}

#[derive(Clone, Debug)]
pub struct MaterialInstanceProperties {
    pub name: String,
    pub material_data: Option<MaterialConstantData>,
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
            material,
            textures: Vec::new(),
        }
    }

    pub fn textures(mut self, textures: &[ResourceHandle<Texture>]) -> Self {
        self.textures.extend_from_slice(textures);

        self
    }

    pub fn add_prop(&mut self, prop: MaterialDataPropData) {
        let material_data = self.material_data.get_or_insert_with(Default::default);

        match prop {
            MaterialDataPropData::DiffuseColor(color) => {
                material_data.diffuse_color = color;
            }
            MaterialDataPropData::EmissionColor(color) => {
                material_data.emission_color = color;
            }
            MaterialDataPropData::SpecularColor(color) => {
                material_data.specular_color = color;
            }
            MaterialDataPropData::SpecularPower(power) => {
                material_data.specular_power = power;
            }
            MaterialDataPropData::DiffuseIndex(index) => {
                material_data.diffuse_index = index;
            }
            MaterialDataPropData::NormalIndex(index) => {
                material_data.normal_index = index;
            }
            MaterialDataPropData::EmissionIndex(index) => {
                material_data.emission_index = index;
            }
            MaterialDataPropData::SpecularIndex(index) => {
                material_data.specular_index = index;
            }
        }
    }

    pub fn prop(mut self, prop: MaterialDataPropData) -> Self {
        self.add_prop(prop);

        self
    }
}

pub struct MaterialInstanceData {
    pub material: ResourceHandle<Material>,
    pub material_binding: MaterialBinding,
}
