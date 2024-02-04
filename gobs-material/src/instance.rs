use std::sync::{Arc, RwLock};

use gobs_vulkan::descriptor::DescriptorSet;

use crate::{texture::Texture, Material};

pub struct MaterialInstance {
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
            material,
            material_ds,
            texture: RwLock::new(texture),
        })
    }
}
