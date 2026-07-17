use gobs_render_hal::{BufferType, Handle, RenderHAL, UniformData as _, UniformPropData};
use gobs_resource::{
    ResourceRegistry, {Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::{
    MaterialInstanceProperties, MaterialProperties,
    data::{MaterialConstantData, MaterialDataLayout, MaterialDataProp},
    resources::{MaterialInstance, MaterialInstanceData},
};

pub struct MaterialInstanceLoader {}

impl MaterialInstanceLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MaterialInstanceLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLoader<MaterialInstance> for MaterialInstanceLoader {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load<'a>(
        &mut self,
        hal: &mut (dyn RenderHAL + 'a),
        handle: &ResourceHandle<MaterialInstance>,
        registry: &mut ResourceRegistry,
    ) -> Result<MaterialInstanceData, ResourceError> {
        let resource = registry.get(handle);
        let properties = &resource.properties;
        let material_handle = properties.material;
        let material_resource = registry.get(&material_handle);
        let material_properties = &material_resource.properties;

        Self::validate_layout(properties, material_properties);

        let material_buffer = self.create_buffer(
            hal,
            properties.name(),
            &material_properties.material_data_layout,
            properties.material_data.as_ref(),
        );

        let data = MaterialInstanceData {
            material: properties.material,
            material_buffer,
            bound: false,
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<MaterialInstance>) {}
}

impl MaterialInstanceLoader {
    fn validate_layout(
        properties: &MaterialInstanceProperties,
        material_properties: &MaterialProperties,
    ) {
        if properties.material_data.is_none()
            && !material_properties.material_data_layout.is_empty()
        {
            tracing::error!("Material instance does not contain material data");
            panic!("Failed to load material instance: {}", &properties.name);
        }
    }

    fn create_buffer(
        &self,
        hal: &mut dyn RenderHAL,
        name: &str,
        material_data_layout: &MaterialDataLayout,
        material_data: Option<&MaterialConstantData>,
    ) -> Option<Handle> {
        let mut data = Vec::new();

        if let Some(material_data) = material_data {
            material_data_layout.copy_data(&mut data, |prop| match prop {
                MaterialDataProp::DiffuseColor => {
                    UniformPropData::Vec4F(material_data.diffuse_color)
                }
                MaterialDataProp::EmissionColor => {
                    UniformPropData::Vec4F(material_data.emission_color)
                }
                MaterialDataProp::SpecularColor => {
                    UniformPropData::Vec4F(material_data.specular_color)
                }
                MaterialDataProp::SpecularPower => {
                    UniformPropData::F32(material_data.specular_power)
                }
            });

            let buffer = hal.create_buffer(name, data.len(), BufferType::Uniform);
            hal.upload_buffer(buffer, &data, 0);

            Some(buffer)
        } else {
            None
        }
    }
}
