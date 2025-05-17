use std::sync::Arc;

use gobs_core::Transform;
use gobs_gfx::GfxBindingGroup;
use gobs_resource::manager::ResourceManager;
use uuid::Uuid;

use crate::{
    GfxContext, MaterialInstance, RenderPass, batch::RenderBatch, materials::MaterialInstanceId,
    resources::MeshData,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RenderableLifetime {
    Static,
    Transient,
}

pub struct RenderObject {
    pub model_id: Uuid,
    pub transform: Transform,
    pub pass: RenderPass,
    pub mesh: MeshData,
    pub material: Option<Arc<MaterialInstance>>,
    pub material_binding: Option<GfxBindingGroup>,
}

impl RenderObject {
    pub fn is_transparent(&self) -> bool {
        if let Some(material) = &self.material {
            material.material.blending_enabled
        } else {
            false
        }
    }

    pub fn material_id(&self) -> Option<MaterialInstanceId> {
        self.material.as_ref().map(|material| material.id)
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
    );
}
