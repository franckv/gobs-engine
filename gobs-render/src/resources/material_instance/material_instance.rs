use gobs_gfx::GfxBindingGroup;
use gobs_resource::resource::{ResourceHandle, ResourceProperties, ResourceType};

use crate::resources::{Material, MaterialInstanceLoader, Texture};

#[derive(Clone, Copy, Debug)]
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
    pub material: ResourceHandle<Material>,
    pub textures: Vec<ResourceHandle<Texture>>,
}

impl ResourceProperties for MaterialInstanceProperties {
    fn name(&self) -> &str {
        &self.name
    }
}

impl MaterialInstanceProperties {
    pub fn new(
        name: &str,
        material: ResourceHandle<Material>,
        textures: Vec<ResourceHandle<Texture>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            material,
            textures,
        }
    }
}

pub struct MaterialInstanceData {
    pub material: ResourceHandle<Material>,
    pub material_binding: Option<GfxBindingGroup>,
    pub texture_binding: Option<GfxBindingGroup>,
    pub bound: bool,
}
