use gobs_core::Transform;
use gobs_resource::manager::ResourceManager;

use crate::{GPUMesh, GfxContext, RenderPass, batch::RenderBatch, materials::MaterialInstanceId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RenderableLifetime {
    Static,
    Transient,
}

pub struct RenderObject {
    pub transform: Transform,
    pub pass: RenderPass,
    pub mesh: GPUMesh,
}

impl RenderObject {
    pub fn is_transparent(&self) -> bool {
        if let Some(material) = &self.mesh.material {
            material.material.blending_enabled
        } else {
            false
        }
    }

    pub fn material_id(&self) -> Option<MaterialInstanceId> {
        self.mesh.material.as_ref().map(|material| material.id)
    }
}

pub trait Renderable {
    fn draw(
        &self,
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
        lifetime: RenderableLifetime,
    );
}
