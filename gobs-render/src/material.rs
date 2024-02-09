mod instance;
mod materials;
mod texture;

pub use instance::MaterialInstance;
pub use materials::{ColorMaterial, DepthMaterial, NormalMaterial, TextureMaterial};
pub use texture::{Texture, TextureType};

use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{descriptor::DescriptorSetPool, pipeline::Pipeline};

use crate::geometry::VertexFlag;

pub type MaterialId = Uuid;

pub enum Material {
    Color(ColorMaterial),
    Depth(DepthMaterial),
    Texture(TextureMaterial),
    Normal(NormalMaterial),
}

impl Material {
    fn ds_pool(&self) -> Option<&RwLock<DescriptorSetPool>> {
        match self {
            Material::Color(_) => None,
            Material::Depth(_) => None,
            Material::Normal(mat) => Some(&mat.material_ds_pool),
            Material::Texture(mat) => Some(&mat.material_ds_pool),
        }
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        match self {
            Material::Color(mat) => mat.vertex_flags,
            Material::Depth(mat) => mat.vertex_flags,
            Material::Normal(mat) => mat.vertex_flags,
            Material::Texture(mat) => mat.vertex_flags,
        }
    }

    pub fn pipeline(&self) -> &Pipeline {
        match self {
            Material::Color(mat) => &mat.pipeline,
            Material::Depth(mat) => &mat.pipeline,
            Material::Normal(mat) => &mat.pipeline,
            Material::Texture(mat) => &mat.pipeline,
        }
    }

    pub fn model_data_layout(&self) -> Arc<UniformLayout> {
        match self {
            Material::Color(mat) => mat.model_data_layout.clone(),
            Material::Depth(mat) => mat.model_data_layout.clone(),
            Material::Normal(mat) => mat.model_data_layout.clone(),
            Material::Texture(mat) => mat.model_data_layout.clone(),
        }
    }
}
