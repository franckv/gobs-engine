use std::sync::Arc;

use uuid::Uuid;

use crate::geometry::Mesh;
use crate::material::MaterialInstance;

pub type ModelId = Uuid;
pub type MaterialIndex = usize;

pub struct Model {
    pub name: String,
    pub id: Uuid,
    pub meshes: Vec<(Arc<Mesh>, MaterialIndex)>,
    pub materials: Vec<Arc<MaterialInstance>>,
}

impl Model {
    pub fn builder(name: &str) -> ModelBuilder {
        ModelBuilder::new(name)
    }
}

pub struct ModelBuilder {
    pub name: String,
    pub meshes: Vec<(Arc<Mesh>, MaterialIndex)>,
    pub materials: Vec<Arc<MaterialInstance>>,
}

impl ModelBuilder {
    pub fn new(name: &str) -> Self {
        ModelBuilder {
            name: name.to_string(),
            meshes: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn mesh(mut self, mesh: Arc<Mesh>, material_idx: MaterialIndex) -> Self {
        self.meshes.push((mesh, material_idx));

        self
    }

    pub fn material(mut self, material: Arc<MaterialInstance>) -> Self {
        self.materials.push(material);

        self
    }

    pub fn materials(mut self, materials: &mut Vec<Arc<MaterialInstance>>) -> Self {
        log::debug!("Create model materials ({:?})", materials);

        self.materials.append(materials);

        self
    }

    pub fn build(self) -> Arc<Model> {
        Arc::new(Model {
            name: self.name,
            id: Uuid::new_v4(),
            meshes: self.meshes,
            materials: self.materials,
        })
    }
}
