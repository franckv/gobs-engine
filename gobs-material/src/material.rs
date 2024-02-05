pub(crate) mod texture_mat;

use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{descriptor::DescriptorSetPool, pipeline::Pipeline};

use crate::{vertex::VertexFlag, TextureMaterial};

pub type MaterialId = Uuid;

pub enum Material {
    Texture(TextureMaterial),
}

impl Material {
    fn ds_pool(&self) -> &RwLock<DescriptorSetPool> {
        match self {
            Material::Texture(mat) => &mat.material_ds_pool,
        }
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        match self {
            Material::Texture(mat) => mat.vertex_flags,
        }
    }

    pub fn pipeline(&self) -> &Pipeline {
        match self {
            Material::Texture(mat) => &mat.pipeline,
        }
    }

    pub fn model_data_layout(&self) -> Arc<UniformLayout> {
        match self {
            Material::Texture(mat) => mat.model_data_layout.clone(),
        }
    }
}
