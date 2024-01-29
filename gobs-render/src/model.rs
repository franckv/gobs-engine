use std::sync::Arc;

use gobs_core::geometry::{
    mesh::{Mesh, Primitive},
    vertex::VertexFlag,
};

use crate::{context::Context, mesh_buffer::MeshBuffer};

pub struct Model {
    pub buffers: Arc<MeshBuffer>,
    pub primitives: Vec<Primitive>,
}

impl Model {
    pub fn new(ctx: &Context, mesh: Arc<Mesh>, vertex_flags: VertexFlag) -> Arc<Self> {
        log::debug!("New model from mesh {}", mesh.name);

        let buffers = MeshBuffer::new(ctx, mesh.clone(), vertex_flags);

        Arc::new(Model {
            buffers,
            primitives: mesh.primitives.clone(),
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        log::debug!("Drop model");
    }
}
