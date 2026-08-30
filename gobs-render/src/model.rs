use std::fmt::Debug;
use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use gobs_core::{Transform, logger};
use gobs_graphics::{Bounded, BoundingBox};
use gobs_render_hal::GfxContext;
use gobs_render_material::RenderFlags;
use gobs_resource::{
    ResourceManager, {ResourceError, ResourceHandle},
};

use crate::{MaterialInstance, Mesh, Renderable, batch::RenderBatch};

pub type ModelId = Uuid;

#[derive(Serialize)]
pub struct Model {
    pub name: Arc<String>,
    pub id: ModelId,
    pub meshes: Vec<(
        ResourceHandle<Mesh>,
        Option<ResourceHandle<MaterialInstance>>,
    )>,
    #[serde(skip)]
    pub bounding_box: BoundingBox,
}

impl Model {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dump(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl Renderable for Arc<Model> {
    fn draw(
        &self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
        bounding_box: Option<BoundingBox>,
        render_flags: RenderFlags,
    ) -> Result<(), ResourceError> {
        if let Some(transform) = transform {
            batch.add_model(
                ctx,
                resource_manager,
                self.clone(),
                transform,
                bounding_box,
                render_flags,
            )?;
        } else {
            tracing::warn!("No transform");
        }

        Ok(())
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model: {}", self.name)
    }
}

impl Bounded for Model {
    fn boundings(&self) -> BoundingBox {
        self.bounding_box
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop Model: {}", &self.name);
    }
}
