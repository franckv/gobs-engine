use gobs_resource::{ResourceHandle, ResourceLifetime, ResourceManager};

use crate::{
    Material, MaterialDataPropData, MaterialInstance, MaterialInstanceProperties, Texture,
};

pub struct RenderMaterialBuilder<'a> {
    name: &'a str,
    resource_manager: &'a mut ResourceManager,
    material: Option<ResourceHandle<Material>>,
    textures: Vec<ResourceHandle<Texture>>,
    material_data: Vec<MaterialDataPropData>,
    lifetime: ResourceLifetime,
}

impl<'a> RenderMaterialBuilder<'a> {
    pub fn new(resource_manager: &'a mut ResourceManager, name: &'a str) -> Self {
        Self {
            name,
            resource_manager,
            material: None,
            textures: Vec::new(),
            material_data: Vec::new(),
            lifetime: ResourceLifetime::Static,
        }
    }

    pub fn from_base(mut self, name: &str) -> Self {
        self.material = Some(self.resource_manager.get_by_name(name).unwrap());

        self
    }

    pub fn with_prop(mut self, prop: MaterialDataPropData) -> Self {
        self.material_data.push(prop);

        self
    }

    pub fn with_textures(mut self, textures: &[ResourceHandle<Texture>]) -> Self {
        self.textures.extend_from_slice(textures);

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

    pub fn build(self) -> ResourceHandle<MaterialInstance> {
        let mut material_instance_properties =
            MaterialInstanceProperties::new(self.name, self.material.unwrap());

        if !self.textures.is_empty() {
            material_instance_properties = material_instance_properties.textures(&self.textures);
        }

        if !self.material_data.is_empty() {
            for prop in self.material_data {
                material_instance_properties = material_instance_properties.prop(prop)
            }
        }

        self.resource_manager
            .add(material_instance_properties, self.lifetime, false)
    }
}
