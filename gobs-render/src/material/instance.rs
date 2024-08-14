use std::fmt::Debug;
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    geometry::VertexFlag,
    material::{Material, Texture},
    GfxBindingGroup, GfxPipeline,
};

pub type MaterialInstanceId = Uuid;

pub struct MaterialInstance {
    pub id: MaterialInstanceId,
    pub material: Arc<Material>,
    pub material_binding: Option<GfxBindingGroup>,
    pub textures: Vec<Texture>,
}

impl MaterialInstance {
    pub(crate) fn new(
        material: Arc<Material>,
        material_binding: Option<GfxBindingGroup>,
        textures: Vec<Texture>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            material,
            material_binding,
            textures,
        })
    }

    pub fn pipeline(&self) -> Arc<GfxPipeline> {
        self.material.pipeline.clone()
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
