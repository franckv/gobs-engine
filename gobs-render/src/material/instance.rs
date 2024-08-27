use std::fmt::Debug;
use std::sync::Arc;

use uuid::Uuid;

use gobs_gfx::Renderer;
use gobs_resource::{geometry::VertexFlag, material::Texture};

use crate::material::Material;

pub type MaterialInstanceId = Uuid;

pub struct MaterialInstance<R: Renderer> {
    pub id: MaterialInstanceId,
    pub material: Arc<Material<R>>,
    pub textures: Vec<Arc<Texture>>,
}

impl<R: Renderer> MaterialInstance<R> {
    pub(crate) fn new(material: Arc<Material<R>>, textures: Vec<Arc<Texture>>) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            material,
            textures,
        })
    }

    pub fn pipeline(&self) -> Arc<R::Pipeline> {
        self.material.pipeline.clone()
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        self.material.vertex_flags
    }
}

impl<R: Renderer> Debug for MaterialInstance<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialInstance")
            .field("id", &self.id)
            .field("material", &self.material.id)
            .finish()
    }
}
