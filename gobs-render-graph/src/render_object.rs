use std::{cmp::Ordering, sync::Arc};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gobs_core::Transform;
use gobs_render_hal::{BindResource, Handle, VertexAttribute};

use crate::data::SceneDataLayout;

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

#[derive(Clone, Debug, Default)]
pub struct MaterialRenderData {
    pub material_render_flags: RenderFlags,
    pub pipeline: Option<Handle>,
    pub material_data: Option<BindResource>,
    pub material_textures: Option<BindResource>,
    pub scene_layout: Option<SceneDataLayout>,
    pub texture_indexing: bool,
}

impl Ord for MaterialRenderData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pipeline
            .cmp(&other.pipeline)
            .then(
                self.material_data
                    .as_ref()
                    .map(|bind| bind.id)
                    .cmp(&other.material_data.as_ref().map(|bind| bind.id)),
            )
            .then(
                self.material_textures
                    .as_ref()
                    .map(|bind| bind.id)
                    .cmp(&other.material_textures.as_ref().map(|bind| bind.id)),
            )
    }
}

impl PartialEq for MaterialRenderData {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for MaterialRenderData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for MaterialRenderData {}

pub struct RenderObject {
    pub model: Arc<String>,
    pub transform: Transform,
    pub vertex_buffer: Handle,
    pub index_buffer: Handle,
    pub index_len: usize,
    pub vertex_attribute: VertexAttribute,
    pub material: MaterialRenderData,
    pub render_flags: RenderFlags,
    pub layer: u32,
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
            .then(self.material.cmp(&other.material))
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
