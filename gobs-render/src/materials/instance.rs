use std::{fmt::Debug, sync::Arc};

use uuid::Uuid;

use gobs_resource::resource::ResourceHandle;

use crate::Texture;
use crate::resources::Material;

pub type MaterialInstanceId = Uuid;

pub struct MaterialInstance {
    pub id: MaterialInstanceId,
    pub material: ResourceHandle<Material>,
    pub textures: Vec<ResourceHandle<Texture>>,
}

impl MaterialInstance {
    pub fn new(
        material: ResourceHandle<Material>,
        textures: Vec<ResourceHandle<Texture>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            material,
            textures,
        })
    }
}

impl Debug for MaterialInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialInstance")
            .field("id", &self.id)
            .field("material", &self.material.id)
            .finish()
    }
}
