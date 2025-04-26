use std::fmt::Debug;
use std::sync::Arc;

use uuid::Uuid;

use gobs_gfx::GfxPipeline;
use gobs_resource::{geometry::VertexFlag, material::Texture};

use crate::materials::Material;

pub type MaterialInstanceId = Uuid;

pub struct MaterialInstance {
    pub id: MaterialInstanceId,
    pub material: Arc<Material>,
    pub textures: Vec<Arc<Texture>>,
}

impl MaterialInstance {
    pub(crate) fn new(material: Arc<Material>, textures: Vec<Arc<Texture>>) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            material,
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
