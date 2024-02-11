use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::geometry::Mesh;
use crate::material::{MaterialInstance, MaterialInstanceId};

pub type ModelId = Uuid;

pub struct Model {
    pub name: String,
    pub id: Uuid,
    pub meshes: Vec<(Arc<Mesh>, MaterialInstanceId)>,
    pub materials: HashMap<MaterialInstanceId, Arc<MaterialInstance>>,
}

impl Model {
    pub fn builder(name: &str) -> ModelBuilder {
        ModelBuilder::new(name)
    }
}

pub struct ModelBuilder {
    pub name: String,
    pub meshes: Vec<(Arc<Mesh>, MaterialInstanceId)>,
    pub materials: HashMap<MaterialInstanceId, Arc<MaterialInstance>>,
}

impl ModelBuilder {
    pub fn new(name: &str) -> Self {
        ModelBuilder {
            name: name.to_string(),
            meshes: Vec::new(),
            materials: HashMap::new(),
        }
    }

    pub fn mesh(mut self, mesh: Arc<Mesh>, material_instance: Arc<MaterialInstance>) -> Self {
        self.meshes.push((mesh, material_instance.id));

        if let Entry::Vacant(entry) = self.materials.entry(material_instance.id) {
            entry.insert(material_instance);
        }

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
