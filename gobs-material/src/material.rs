pub(crate) mod normal_mat;
pub(crate) mod texture_mat;

use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{descriptor::DescriptorSetPool, pipeline::Pipeline};

use crate::{vertex::VertexFlag, TextureMaterial};

use self::normal_mat::NormalMaterial;

pub type MaterialId = Uuid;

pub enum Material {
    Texture(TextureMaterial),
    Normal(NormalMaterial),
}

impl Material {
    fn ds_pool(&self) -> &RwLock<DescriptorSetPool> {
        match self {
            Material::Normal(mat) => &mat.material_ds_pool,
            Material::Texture(mat) => &mat.material_ds_pool,
        }
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        match self {
            Material::Normal(mat) => mat.vertex_flags,
            Material::Texture(mat) => mat.vertex_flags,
        }
    }

    pub fn pipeline(&self) -> &Pipeline {
        match self {
            Material::Normal(mat) => &mat.pipeline,
            Material::Texture(mat) => &mat.pipeline,
        }
    }

    pub fn model_data_layout(&self) -> Arc<UniformLayout> {
        match self {
            Material::Normal(mat) => mat.model_data_layout.clone(),
            Material::Texture(mat) => mat.model_data_layout.clone(),
        }
    }
}
