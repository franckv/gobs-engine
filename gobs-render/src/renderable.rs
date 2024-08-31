use gobs_core::Transform;

use crate::{
    batch::RenderBatch, context::Context, material::MaterialInstanceId, resources::GPUMesh,
    RenderPass,
};

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
        if let Some(material) = &self.mesh.material {
            Some(material.id)
        } else {
            None
        }
    }
}

pub trait Renderable {
    fn draw(
        &self,
        ctx: &Context,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
        lifetime: RenderableLifetime,
    );
}
