use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{descriptor::DescriptorSet, pipeline::Pipeline};

use crate::{texture::Texture, Material};

pub type MaterialInstanceId = Uuid;

pub struct MaterialInstance {
    pub id: MaterialInstanceId,
    pub material: Arc<Material>,
    pub material_ds: DescriptorSet,
    pub texture: RwLock<Texture>,
}

impl MaterialInstance {
    pub(crate) fn new(
        material: Arc<Material>,
        material_ds: DescriptorSet,
        texture: Texture,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            material,
            material_ds,
            texture: RwLock::new(texture),
        })
    }

    pub fn pipeline(&self) -> &Pipeline {
        self.material.pipeline()
    }

    pub fn model_data_layout(&self) -> Arc<UniformLayout> {
        self.material.model_data_layout()
    }
}
