use std::sync::Arc;

use gobs_core::Transform;
use gobs_gfx::{GfxBindingGroup, GfxPipeline, Pipeline, PipelineId};
use gobs_resource::manager::ResourceManager;
use uuid::Uuid;

use crate::{GfxContext, RenderPass, batch::RenderBatch, resources::MeshData};

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
    pub pipeline: Option<Arc<GfxPipeline>>,
    pub is_transparent: bool,
    pub bind_groups: Vec<GfxBindingGroup>,
}

impl RenderObject {
    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub fn pipeline_id(&self) -> PipelineId {
        if let Some(pipeline) = &self.pipeline {
            pipeline.id()
        } else {
            PipelineId::default()
        }
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
