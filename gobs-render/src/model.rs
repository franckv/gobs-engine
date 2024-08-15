use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use gobs_resource::geometry::{Bounded, BoundingBox, Mesh};

use crate::material::{MaterialInstance, MaterialInstanceId};

pub type ModelId = Uuid;

#[derive(Serialize)]
pub struct Model {
    pub name: String,
    pub id: ModelId,
    pub meshes: Vec<(Arc<Mesh>, MaterialInstanceId)>,
    #[serde(skip)]
    pub materials: HashMap<MaterialInstanceId, Arc<MaterialInstance>>,
}

impl Model {
    pub fn builder(name: &str) -> ModelBuilder {
        ModelBuilder::new(name)
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self).unwrap();

        write!(f, "Model: {}", json)
    }
}

impl Bounded for Model {
    fn boundings(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        for (mesh, _) in &self.meshes {
            bounding_box.extends_box(mesh.boundings());
        }

        bounding_box
    }
}

pub struct ModelBuilder {
    pub name: String,
    pub id: ModelId,
    pub meshes: Vec<(Arc<Mesh>, MaterialInstanceId)>,
    pub materials: HashMap<MaterialInstanceId, Arc<MaterialInstance>>,
}

impl ModelBuilder {
    pub fn new(name: &str) -> Self {
        ModelBuilder {
            name: name.to_string(),
            id: Uuid::new_v4(),
            meshes: Vec::new(),
            materials: HashMap::new(),
        }
    }

    pub fn id(mut self, model_id: ModelId) -> Self {
        self.id = model_id;

        self
    }

    pub fn mesh(
        mut self,
        mesh: Arc<Mesh>,
        material_instance: Option<Arc<MaterialInstance>>,
    ) -> Self {
        if let Some(material_instance) = material_instance {
            self.meshes.push((mesh, material_instance.id));

            if let Entry::Vacant(entry) = self.materials.entry(material_instance.id) {
                entry.insert(material_instance);
            }
        } else {
            self.meshes.push((mesh, MaterialInstanceId::nil()))
        }

        self
    }

    pub fn build(self) -> Arc<Model> {
        Arc::new(Model {
            name: self.name,
            id: self.id,
            meshes: self.meshes,
            materials: self.materials,
        })
    }
}
