use std::cmp::Ordering;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gobs_core::Transform;
use gobs_render_hal::{BindResource, Handle, VertexAttribute};

pub type MaterialId = Uuid;
pub type MaterialInstanceId = Uuid;
pub type MeshId = Uuid;
pub type PassId = Uuid;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct RenderFlags: u32 {
        const ENTITY = 1 << 0;
        const TRANSPARENT = 1 << 1;
        const OPAQUE = 1 << 2;
        const UI = 1 << 3;
        const SELECTED = 1 << 4;
        const BOUNDS = 1 << 5;
    }
}

pub struct RenderObject {
    pub model: String,
    pub transform: Transform,
    pub vertex_buffer: Handle,
    pub index_buffer: Handle,
    pub index_len: usize,
    pub vertex_attribute: VertexAttribute,
    pub pipeline: Option<Handle>,
    pub material_data: Option<BindResource>,
    pub material_textures: Option<BindResource>,
    pub layer: u32,
    pub render_flags: RenderFlags,
}

impl RenderObject {
    pub fn is_transparent(&self) -> bool {
        self.render_flags.contains(RenderFlags::TRANSPARENT)
    }
}

// sort order: pass, transparent, material, model
impl Ord for RenderObject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer
            .cmp(&other.layer)
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
