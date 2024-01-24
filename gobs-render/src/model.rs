use std::sync::Arc;

use crate::mesh::{MeshBuffer, MeshSurface};

pub struct Model {
    pub buffers: Arc<MeshBuffer>,
    pub surfaces: Vec<MeshSurface>,
}

impl Model {
    pub fn new(buffers: Arc<MeshBuffer>) -> Self {
        Model {
            buffers,
            surfaces: Vec::new(),
        }
    }

    pub fn add_surface(&mut self, offset: usize, len: usize) {
        self.surfaces.push(MeshSurface { offset, len });
    }
}
