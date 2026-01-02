use gobs_render_hal::{BufferType, Handle, RenderHAL};
use gobs_resource::{
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::{
    MaterialConstantData, MaterialDataLayout, UniformData,
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
    fn load(
        &mut self,
        hal: &mut Box<dyn RenderHAL>,
        handle: &ResourceHandle<MaterialInstance>,
        registry: &mut ResourceRegistry,
    ) -> Result<MaterialInstanceData, ResourceError> {
        let resource = registry.get(handle);
        let properties = &resource.properties;

        let material_buffer = self.create_buffer(
            hal.as_mut(),
            properties.name(),
            &properties.material_data_layout,
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
    fn create_buffer(
        &self,
        hal: &mut dyn RenderHAL,
        name: &str,
        material_data_layout: &MaterialDataLayout,
        material_data: Option<&MaterialConstantData>,
    ) -> Option<Handle> {
        let mut data = Vec::new();

        if let Some(material_data) = material_data {
            material_data_layout.copy_data(None, material_data, &mut data);

            let buffer = hal.create_buffer(name, data.len(), BufferType::Uniform);
            hal.upload_buffer(buffer, &data, 0);

            Some(buffer)
        } else {
            None
        }
    }
}
