use std::sync::Arc;

use gobs_core::entity::uniform::UniformLayout;
use gobs_material::Material;
use gobs_render::context::Context;

use crate::{
    geometry::{
        mesh::{Mesh, Primitive},
        vertex::VertexFlag,
    },
    mesh_buffer::MeshBuffer,
};

pub struct Model {
    pub buffers: Arc<MeshBuffer>,
    pub materials: Vec<Material>,
    pub primitives: Vec<Primitive>,
    pub model_data_layout: Arc<UniformLayout>,
}

impl Model {
    pub fn new(
        ctx: &Context,
        mesh: Arc<Mesh>,
        model_data_layout: Arc<UniformLayout>,
        vertex_flags: VertexFlag,
        materials: Vec<Material>,
    ) -> Arc<Self> {
        log::debug!("New model from mesh {}", mesh.name);

        let buffers = MeshBuffer::new(ctx, mesh.clone(), vertex_flags);

        Arc::new(Model {
            buffers,
            materials,
            primitives: mesh.primitives.clone(),
            model_data_layout,
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        log::debug!("Drop model");
    }
}
