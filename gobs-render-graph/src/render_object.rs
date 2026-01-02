use std::cmp::Ordering;

use uuid::Uuid;

use gobs_core::Transform;
use gobs_render_hal::{BindResource, Handle};

pub type MaterialId = Uuid;
pub type MaterialInstanceId = Uuid;
pub type MeshId = Uuid;
pub type PassId = Uuid;

pub struct RenderObject {
    pub transform: Transform,
    pub pass_id: PassId,
    pub vertex_buffer: Handle,
    pub index_buffer: Handle,
    pub index_len: usize,
    pub pipeline: Option<Handle>,
    pub is_transparent: bool,
    pub material_data: Option<BindResource>,
    pub material_textures: Option<BindResource>,
    pub layer: u32,
}

impl RenderObject {
    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }
}

// sort order: pass, transparent, material, model
impl Ord for RenderObject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer
            .cmp(&other.layer)
            .then(self.pass_id.cmp(&other.pass_id))
            .then(self.is_transparent().cmp(&other.is_transparent()))
            .then(self.pipeline.cmp(&other.pipeline))
            .then(self.index_buffer.cmp(&other.index_buffer))
    }
}

impl PartialEq for RenderObject {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for RenderObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for RenderObject {}
