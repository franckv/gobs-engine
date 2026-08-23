use gobs_core::logger;
use gobs_render_hal::RenderHAL;
use gobs_resource::{
    ResourceRegistry, {ResourceError, ResourceHandle, ResourceLoader},
};

use crate::{
    material_system::MaterialSystem,
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

        let material_binding =
            MaterialSystem::get_material_binding(hal, properties, &material_properties);

        let data = MaterialInstanceData {
            material: properties.material,
            material_binding,
        };

        Ok(data)
    }

    fn unload<'a>(&mut self, hal: &mut (dyn RenderHAL + 'a), data: MaterialInstanceData) {
        MaterialSystem::destroy_material_binding(hal, data.material_binding);
    }

    fn flush(&mut self) {}
}

impl MaterialInstanceLoader {}
