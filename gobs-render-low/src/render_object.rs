use std::sync::Arc;

use uuid::Uuid;

use gobs_core::Transform;
use gobs_gfx::{GfxBindingGroup, GfxBuffer, GfxPipeline, Pipeline, PipelineId};

pub struct RenderObject {
    pub model_id: Uuid,
    pub transform: Transform,
    pub pass_id: Uuid,
    pub vertex_buffer: Arc<GfxBuffer>,
    pub vertices_offset: u64,
    pub vertices_len: usize,
    pub vertices_count: usize,
    pub index_buffer: Arc<GfxBuffer>,
    pub indices_offset: usize,
    pub indices_len: usize,
    pub pipeline: Option<Arc<GfxPipeline>>,
    pub is_transparent: bool,
    pub bind_groups: Vec<GfxBindingGroup>,
    pub material_instance_id: Uuid,
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
