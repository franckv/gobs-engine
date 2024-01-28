use std::sync::Arc;

use gobs_core::geometry::{mesh::Mesh, vertex::VertexFlag};

use crate::{
    context::Context,
    mesh::{MeshBuffer, MeshSurface},
};

pub struct Model {
    pub buffers: Arc<MeshBuffer>,
    pub surfaces: Vec<MeshSurface>,
}

impl Model {
    pub fn new(ctx: &Context, mesh: Arc<Mesh>, vertex_flags: VertexFlag) -> Self {
        let buffers = MeshBuffer::new(ctx, mesh.clone(), vertex_flags);
        let mut surfaces = Vec::new();

        let mut offset = 0;
        for p in &mesh.primitives {
            surfaces.push(MeshSurface {
                offset,
                len: p.indices.len(),
            });
            offset += p.indices.len();
        }

        Model { buffers, surfaces }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        log::debug!("Drop model");
    }
}
