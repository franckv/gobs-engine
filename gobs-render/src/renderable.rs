use std::sync::Arc;

use gobs_core::Transform;

use crate::{
    batch::RenderBatch, context::Context, material::MaterialInstanceId, pass::RenderPass,
    resources::GPUMesh,
};

pub struct RenderObject {
    pub transform: Transform,
    pub pass: Arc<dyn RenderPass>,
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
    fn resize(&mut self, width: u32, height: u32);
    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch);
}
