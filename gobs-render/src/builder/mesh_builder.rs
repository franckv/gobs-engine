use std::sync::Arc;

use gobs_render_hal::VertexAttribute;
use gobs_resource::{ResourceHandle, ResourceLifetime, ResourceManager};

use crate::{Material, MaterialInstance, Mesh, MeshGeometry, MeshProperties};

pub struct RenderMeshBuilder<'a> {
    name: &'a str,
    resource_manager: &'a mut ResourceManager,
    geometry: Option<Arc<MeshGeometry>>,
    bytes: Option<(Vec<u8>, Vec<u32>)>,
    vertex_attributes: VertexAttribute,
    layer: u32,
    lifetime: ResourceLifetime,
}

impl<'a> RenderMeshBuilder<'a> {
    pub fn new(resource_manager: &'a mut ResourceManager, name: &'a str) -> Self {
        Self {
            name,
            resource_manager,
            geometry: None,
            bytes: None,
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

    pub fn with_bytes(mut self, vertices: Vec<u8>, indices: Vec<u32>) -> Self {
        self.bytes = Some((vertices, indices));

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
        let properties = match (self.geometry, self.bytes) {
            (None, Some((vertices, indices))) => MeshProperties::with_bytes(
                self.name,
                vertices,
                indices,
                self.vertex_attributes,
                self.layer,
            ),
            (Some(geometry), None) => MeshProperties::with_geometry(
                self.name,
                geometry,
                self.vertex_attributes,
                self.layer,
            ),
            _ => panic!("Invalid mesh data"),
        };

        self.resource_manager.add(properties, self.lifetime, false)
    }

    fn get_vertex_attributes(
        &self,
        material_instance: ResourceHandle<MaterialInstance>,
    ) -> VertexAttribute {
        let material_instance = self
            .resource_manager
            .get::<MaterialInstance>(&material_instance)
            .expect("Material instance not registered");
        let material = self
            .resource_manager
            .get::<Material>(&material_instance.properties.material)
            .expect("Material not registered");

        material.properties.pipeline_properties.vertex_attributes
    }
}
