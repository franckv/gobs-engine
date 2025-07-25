use gobs_render_graph::{Pipeline, PipelineProperties};
use gobs_resource::{
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader},
};

use crate::resources::{MaterialData, material::Material};

pub struct MaterialLoader {}

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
    fn load(
        &mut self,
        handle: &ResourceHandle<Material>,
        _parameter: &(),
        registry: &mut ResourceRegistry,
    ) -> Result<MaterialData, ResourceError> {
        let (pipeline_properties, lifetime) = {
            let resource = registry.get(handle);
            (
                PipelineProperties::Graphics(resource.properties.pipeline_properties.clone()),
                resource.lifetime,
            )
        };

        let pipeline_handle = registry.add::<Pipeline>(pipeline_properties, lifetime);

        Ok(MaterialData {
            pipeline: pipeline_handle,
        })
    }

    fn unload(&mut self, _resource: Resource<Material>) {}
}
