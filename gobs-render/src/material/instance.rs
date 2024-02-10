use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{descriptor::DescriptorSet, pipeline::Pipeline};

use crate::{
    geometry::VertexFlag,
    material::{Material, Texture},
};

pub type MaterialInstanceId = Uuid;

pub struct MaterialInstance {
    pub id: MaterialInstanceId,
    pub material: Arc<Material>,
    pub material_ds: Option<DescriptorSet>,
    _texture: Vec<Arc<RwLock<Texture>>>,
}

impl MaterialInstance {
    pub(crate) fn new(
        material: Arc<Material>,
        material_ds: Option<DescriptorSet>,
        mut textures: Vec<Texture>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            material,
            material_ds,
            _texture: textures
                .drain(..)
                .map(|texture| Arc::new(RwLock::new(texture)))
                .collect(),
        })
    }

    pub fn pipeline(&self) -> &Pipeline {
        &self.material.pipeline
    }

    pub fn model_data_layout(&self) -> &UniformLayout {
        &self.material.model_data_layout
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        self.material.vertex_flags
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
