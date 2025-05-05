use std::sync::Arc;

use gobs_gfx::{GfxBindingGroup, GfxBuffer};

use crate::{MaterialInstance, Model};

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Debug)]
pub struct GPUMesh {
    pub model: Arc<Model>,
    pub ty: PrimitiveType,
    pub material: Option<Arc<MaterialInstance>>,
    pub material_binding: Option<GfxBindingGroup>,
    pub vertex_buffer: Arc<GfxBuffer>,
    pub index_buffer: Arc<GfxBuffer>,
    pub vertices_offset: u64,
    pub indices_offset: usize,
    pub indices_len: usize,
}

impl Clone for GPUMesh {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            ty: self.ty,
            material: self.material.clone(),
            material_binding: self.material_binding.clone(),
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            vertices_offset: self.vertices_offset,
            indices_offset: self.indices_offset,
            indices_len: self.indices_len,
        }
    }
}
