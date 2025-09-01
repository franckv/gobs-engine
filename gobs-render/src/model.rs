use std::fmt::Debug;
use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use gobs_core::{Transform, logger};
use gobs_render_graph::RenderPass;
use gobs_render_resources::{MaterialInstance, Mesh, MeshProperties};
use gobs_resource::{
    geometry::{Bounded, BoundingBox, MeshGeometry},
    manager::ResourceManager,
    resource::{ResourceError, ResourceHandle, ResourceLifetime},
};

use crate::{Renderable, batch::RenderBatch};

pub type ModelId = Uuid;

#[derive(Serialize)]
pub struct Model {
    pub name: String,
    pub id: ModelId,
    pub meshes: Vec<(
        ResourceHandle<Mesh>,
        Option<ResourceHandle<MaterialInstance>>,
    )>,
    #[serde(skip)]
    pub bounding_box: BoundingBox,
}

impl Model {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn builder(name: &str) -> ModelBuilder {
        ModelBuilder::new(name)
    }

    pub fn dump(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl Renderable for Arc<Model> {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn draw(
        &self,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
    ) -> Result<(), ResourceError> {
        if let Some(transform) = transform {
            batch.add_model(resource_manager, self.clone(), transform, pass.clone())?;
        } else {
            tracing::warn!("No transform");
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
        tracing::debug!(target: logger::MEMORY, "Drop Model: {}", &self.name);
    }
}

pub struct ModelBuilder {
    pub name: String,
    pub id: ModelId,
    pub layer: u32,
    pub meshes: Vec<(
        ResourceHandle<Mesh>,
        Option<ResourceHandle<MaterialInstance>>,
    )>,
    pub bounding_box: BoundingBox,
}

impl ModelBuilder {
    pub fn new(name: &str) -> Self {
        ModelBuilder {
            name: name.to_string(),
            id: ModelId::new_v4(),
            layer: 0,
            meshes: Vec::new(),
            bounding_box: BoundingBox::default(),
        }
    }

    pub fn id(mut self, model_id: ModelId) -> Self {
        self.id = model_id;

        self
    }

    pub fn layer(mut self, layer: u32) -> Self {
        self.layer = layer;

        self
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn mesh(
        mut self,
        mesh: Arc<MeshGeometry>,
        material_instance: Option<ResourceHandle<MaterialInstance>>,
        resource_manager: &mut ResourceManager,
        lifetime: ResourceLifetime,
    ) -> Self {
        self.bounding_box.extends_box(mesh.boundings());

        let mesh_handle = resource_manager.add(
            MeshProperties::with_geometry(mesh, self.layer),
            lifetime,
            false,
        );

        if let Some(material_instance) = material_instance {
            self.meshes.push((mesh_handle, Some(material_instance)));
        } else {
            self.meshes.push((mesh_handle, None))
        }

        self
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn build(self) -> Arc<Model> {
        tracing::debug!(target: logger::RESOURCES, "Load model {} ({} meshes)", self.name, self.meshes.len());

        Arc::new(Model {
            name: self.name,
            id: self.id,
            meshes: self.meshes,
            bounding_box: self.bounding_box,
        })
    }
}
