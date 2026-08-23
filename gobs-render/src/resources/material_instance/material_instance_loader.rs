use gobs_core::logger;
use gobs_render_hal::{AttributeData, BufferType, Handle, RenderHAL, UniformData as _};
use gobs_resource::{
    ResourceRegistry, {ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::{
    MaterialDataPropData, MaterialInstanceProperties, MaterialProperties,
    data::{MaterialConstantData, MaterialDataLayout, MaterialDataProp, TextureDataProp},
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
        let material_properties = {
            let resource = registry.get(handle);
            let properties = &resource.properties;
            let material_handle = properties.material;
            let material_resource = registry.get(&material_handle);
            let material_properties = &material_resource.properties;

            material_properties.clone()
        };

        tracing::debug!(target: logger::RESOURCES, "Load material instance with layout {:?}", &material_properties.material_data_layout);

        let properties = {
            let resource = registry.get_mut(handle);

            &mut resource.properties
        };

        Self::update_textures_index(hal, properties, &material_properties);

        let material_buffer = self.create_buffer(
            hal,
            properties.name(),
            &material_properties.material_data_layout,
            properties.material_data.as_ref(),
        );

        let data = MaterialInstanceData {
            material: properties.material,
            material_buffer,
        };

        Ok(data)
    }

    fn unload<'a>(&mut self, hal: &mut (dyn RenderHAL + 'a), data: MaterialInstanceData) {
        if let Some(buffer) = data.material_buffer {
            hal.destroy_buffer(buffer);
        }
    }

    fn flush(&mut self) {}
}

impl MaterialInstanceLoader {
    #[cfg(debug_assertions)]
    fn validate_layout(
        properties: &MaterialInstanceProperties,
        material_properties: &MaterialProperties,
    ) {
        if properties.material_data.is_none()
            && !material_properties.material_data_layout.is_empty()
        {
            tracing::error!(target: logger::RESOURCES, "Material instance does not contain material data");
            panic!("Failed to load material instance: {}", properties.name);
        }
    }

    fn update_textures_index(
        hal: &mut dyn RenderHAL,
        properties: &mut MaterialInstanceProperties,
        material_properties: &MaterialProperties,
    ) {
        #[cfg(debug_assertions)]
        Self::validate_layout(properties, material_properties);

        let layout = &material_properties.texture_data_layout;

        if layout.texture_indexing {
            for &texture_prop in &layout.layout {
                let index = hal.allocate_texture_index() as u32;

                tracing::debug!(target: logger::RESOURCES, "Alloc texture index {} for {:?}", index, texture_prop);

                let prop = match texture_prop {
                    TextureDataProp::Diffuse => MaterialDataPropData::DiffuseIndex(index),
                    TextureDataProp::Normal => MaterialDataPropData::NormalIndex(index),
                    TextureDataProp::Emission => MaterialDataPropData::EmissionIndex(index),
                    TextureDataProp::Specular => MaterialDataPropData::SpecularIndex(index),
                };

                properties.add_prop(prop);
            }
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
                MaterialDataProp::DiffuseColor => AttributeData::Vec4F(material_data.diffuse_color),
                MaterialDataProp::EmissionColor => {
                    AttributeData::Vec4F(material_data.emission_color)
                }
                MaterialDataProp::SpecularColor => {
                    AttributeData::Vec4F(material_data.specular_color)
                }
                MaterialDataProp::SpecularPower => AttributeData::F32(material_data.specular_power),
                MaterialDataProp::DiffuseIndex => AttributeData::U32(material_data.diffuse_index),
                MaterialDataProp::NormalIndex => AttributeData::U32(material_data.normal_index),
                MaterialDataProp::EmissionIndex => AttributeData::U32(material_data.emission_index),
                MaterialDataProp::SpecularIndex => AttributeData::U32(material_data.specular_index),
            });

            tracing::debug!(target: logger::RESOURCES, "Create material data buffer {:?}", &data);

            let buffer = hal.create_buffer(name, data.len(), BufferType::Uniform);
            hal.upload_buffer(buffer, &data, 0);

            Some(buffer)
        } else {
            None
        }
    }
}
