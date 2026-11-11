use std::sync::Arc;

use gobs_graphics::{Bounded as _, BoundingBox, VertexAttribute};
use gobs_render_material::Material;
use gobs_resource::{ResourceHandle, SharedResourceManager};

use crate::{MaterialInstance, Mesh, Model, ModelId, resources::MeshPath};

pub struct RenderModelBuilder<'a> {
    name: &'a str,
    resource_manager: SharedResourceManager,
    meshes: Vec<(
        ResourceHandle<Mesh>,
        Option<ResourceHandle<MaterialInstance>>,
    )>,
    bounding_box: BoundingBox,
}

impl<'a> RenderModelBuilder<'a> {
    pub fn new(resource_manager: SharedResourceManager, name: &'a str) -> Self {
        Self {
            name,
            resource_manager,
            meshes: Vec::new(),
            bounding_box: BoundingBox::default(),
        }
    }

    pub fn with_mesh(mut self, mesh: ResourceHandle<Mesh>) -> Self {
        {
            let resource_manager = &self.resource_manager.lock();

            let mesh_properties = &resource_manager
                .get(&mesh)
                .expect("Mesh not registered")
                .properties;

            match &mesh_properties.path {
                MeshPath::Mesh(geometry) => {
                    self.bounding_box.extends_box(geometry.boundings());
                }
                // TODO: not needed for UI
                MeshPath::Bytes(_) => {}
                _ => unimplemented!(),
            }

            self.meshes.push((mesh, None));
        }

        self
    }

    pub fn with_material(mut self, material: ResourceHandle<MaterialInstance>) -> Self {
        let vertex_attributes = self.get_vertex_attributes(material);

        {
            let resource_manager = &self.resource_manager.lock();

            if let Some((mesh, material_instance)) = self.meshes.last_mut() {
                *material_instance = Some(material);

                let mesh_properties = &resource_manager
                    .get(mesh)
                    .expect("Mesh not registered")
                    .properties;
                debug_assert_eq!(mesh_properties.vertex_attributes, vertex_attributes);
            } else {
                panic!("Missing mesh in model");
            }
        }
        self
    }

    pub fn build(self) -> Arc<Model> {
        Arc::new(Model {
            name: Arc::new(self.name.to_string()),
            id: ModelId::new_v4(),
            meshes: self.meshes,
            bounding_box: self.bounding_box,
        })
    }

    fn get_vertex_attributes(
        &self,
        material_instance: ResourceHandle<MaterialInstance>,
    ) -> VertexAttribute {
        let resource_manager = self.resource_manager.lock();

        let material_instance = resource_manager
            .get::<MaterialInstance>(&material_instance)
            .expect("Material instance not registered");
        let material = resource_manager
            .get::<Material>(&material_instance.properties.material)
            .expect("Material not registered");

        material.properties.pipeline_properties.vertex_attributes
    }
}
