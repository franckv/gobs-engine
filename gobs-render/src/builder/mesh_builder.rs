use std::sync::Arc;

use gobs_render_hal::VertexAttribute;
use gobs_resource::{ResourceHandle, ResourceLifetime, ResourceManager};

use crate::{Material, MaterialInstance, Mesh, MeshGeometry, MeshProperties};

pub struct RenderMeshBuilder<'a> {
    resource_manager: &'a mut ResourceManager,
    geometry: Option<Arc<MeshGeometry>>,
    vertex_attributes: VertexAttribute,
    layer: u32,
    lifetime: ResourceLifetime,
}

impl<'a> RenderMeshBuilder<'a> {
    pub fn new(resource_manager: &'a mut ResourceManager) -> Self {
        Self {
            resource_manager,
            geometry: None,
            vertex_attributes: VertexAttribute::POSITION
                | VertexAttribute::COLOR
                | VertexAttribute::TEXTURE
                | VertexAttribute::NORMAL
                | VertexAttribute::TANGENT
                | VertexAttribute::BITANGENT,
            layer: 0,
            lifetime: ResourceLifetime::Static,
        }
    }

    pub fn with_geometry(mut self, geometry: Arc<MeshGeometry>) -> Self {
        self.geometry = Some(geometry);

        self
    }

    pub fn with_layer(mut self, layer: u32) -> Self {
        self.layer = layer;

        self
    }

    pub fn transient(mut self, transient: bool) -> Self {
        if transient {
            self.lifetime = ResourceLifetime::Transient;
        } else {
            self.lifetime = ResourceLifetime::Static;
        }

        self
    }

    pub fn for_material(mut self, material: ResourceHandle<MaterialInstance>) -> Self {
        self.vertex_attributes = self.get_vertex_attributes(material);

        self
    }

    pub fn build(self) -> ResourceHandle<Mesh> {
        self.resource_manager.add(
            MeshProperties::with_geometry(
                self.geometry.unwrap(),
                self.vertex_attributes,
                self.layer,
            ),
            self.lifetime,
            false,
        )
    }

    fn get_vertex_attributes(
        &self,
        material_instance: ResourceHandle<MaterialInstance>,
    ) -> VertexAttribute {
        let material_instance = self
            .resource_manager
            .get::<MaterialInstance>(&material_instance);
        let material = self
            .resource_manager
            .get::<Material>(&material_instance.properties.material);

        material.properties.pipeline_properties.vertex_attributes
    }
}
