use std::sync::Arc;

use gobs_core::entity::uniform::UniformLayout;
use gobs_material::Material;

use crate::mesh::Mesh;

pub struct Model {
    pub mesh: Arc<Mesh>,
    pub materials: Vec<Arc<Material>>,
    pub model_data_layout: Arc<UniformLayout>,
}

impl Model {
    pub fn new(
        mesh: Arc<Mesh>,
        model_data_layout: Arc<UniformLayout>,
        materials: Vec<Arc<Material>>,
    ) -> Arc<Self> {
        log::debug!("New model from mesh {}", mesh.name);

        Arc::new(Model {
            mesh,
            materials,
            model_data_layout,
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        log::debug!("Drop model");
    }
}
