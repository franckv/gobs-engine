use gobs_core::logger;
use gobs_render_hal::RenderHAL;
use gobs_resource::{
    ResourceRegistry, {ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::{
    MaterialProperties,
    resources::{MaterialData, Pipeline, PipelineProperties, material::Material},
};

pub struct MaterialLoader;

impl MaterialLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MaterialLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLoader<Material> for MaterialLoader {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load<'a>(
        &mut self,
        _hal: &mut (dyn RenderHAL + 'a),
        handle: &ResourceHandle<Material>,
        registry: &mut ResourceRegistry,
    ) -> Result<MaterialData, ResourceError> {
        let (pipeline_properties, lifetime) = {
            let resource = registry.get(handle)?;
            tracing::info!(target: logger::RESOURCES, "Load material resource {} with indexing: textures={}, data={}",
                resource.properties.name(),
                resource.properties.texture_data_layout.texture_indexing,
                resource.properties.material_data_layout.material_indexing);

            (
                PipelineProperties::Graphics(resource.properties.pipeline_properties.clone()),
                resource.lifetime,
            )
        };

        let pipeline_handle = registry.add::<Pipeline>(pipeline_properties, lifetime, false);

        Ok(MaterialData {
            pipeline: pipeline_handle,
        })
    }

    fn unload<'a>(
        &mut self,
        _hal: &mut (dyn RenderHAL + 'a),
        _data: MaterialData,
        _properties: MaterialProperties,
    ) {
    }

    fn flush(&mut self) {}
}
