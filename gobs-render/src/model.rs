use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::sync::Arc;

use gobs_render_low::RenderError;
use serde::Serialize;
use uuid::Uuid;

use gobs_core::Transform;
use gobs_render_graph::RenderPass;
use gobs_resource::{
    geometry::{Bounded, BoundingBox, MeshGeometry},
    manager::ResourceManager,
    resource::{ResourceHandle, ResourceLifetime},
};

use crate::{
    Mesh, Renderable,
    batch::RenderBatch,
    materials::{MaterialInstance, MaterialInstanceId},
    resources::MeshProperties,
};

pub type ModelId = Uuid;

#[derive(Serialize)]
pub struct Model {
    pub name: String,
    pub id: ModelId,
    pub meshes: Vec<(ResourceHandle<Mesh>, MaterialInstanceId)>,
    #[serde(skip)]
    pub materials: HashMap<MaterialInstanceId, Arc<MaterialInstance>>,
    pub bounding_box: BoundingBox,
}

impl Model {
    pub fn builder(name: &str) -> ModelBuilder {
        ModelBuilder::new(name)
    }

    pub fn dump(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl Renderable for Arc<Model> {
    fn draw(
        &self,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
    ) -> Result<(), RenderError> {
        if let Some(transform) = transform {
            batch.add_model(resource_manager, self.clone(), transform, pass.clone())?;
        }

        Ok(())
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model: {}", self.name)
    }
}

impl Bounded for Model {
    fn boundings(&self) -> BoundingBox {
        self.bounding_box
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop Model: {}", &self.name);
    }
}

pub struct ModelBuilder {
    pub name: String,
    pub id: ModelId,
    pub meshes: Vec<(ResourceHandle<Mesh>, MaterialInstanceId)>,
    pub materials: HashMap<MaterialInstanceId, Arc<MaterialInstance>>,
    pub bounding_box: BoundingBox,
}

impl ModelBuilder {
    pub fn new(name: &str) -> Self {
        ModelBuilder {
            name: name.to_string(),
            id: Uuid::new_v4(),
            meshes: Vec::new(),
            materials: HashMap::new(),
            bounding_box: BoundingBox::default(),
        }
    }

    pub fn id(mut self, model_id: ModelId) -> Self {
        self.id = model_id;

        self
    }

    pub fn mesh(
        mut self,
        mesh: Arc<MeshGeometry>,
        material_instance: Option<Arc<MaterialInstance>>,
        resource_manager: &mut ResourceManager,
        lifetime: ResourceLifetime,
    ) -> Self {
        self.bounding_box.extends_box(mesh.boundings());

        let handle = resource_manager.add(MeshProperties::with_geometry("mesh", mesh), lifetime);

        if let Some(material_instance) = material_instance {
            self.meshes.push((handle, material_instance.id));

            if let Entry::Vacant(entry) = self.materials.entry(material_instance.id) {
                entry.insert(material_instance);
            }
        } else {
            self.meshes.push((handle, MaterialInstanceId::nil()))
        }

        self
    }

    pub fn build(self, _resource_manager: &mut ResourceManager) -> Arc<Model> {
        Arc::new(Model {
            name: self.name,
            id: self.id,
            meshes: self.meshes,
            materials: self.materials,
            bounding_box: self.bounding_box,
        })
    }
}
