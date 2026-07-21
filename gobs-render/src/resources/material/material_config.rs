use std::collections::HashMap;

use serde::Deserialize;

use gobs_core::{ImageFormat, logger};
use gobs_render_hal::{
    AlignMode, BlendMode, CullMode, ObjectDataLayout, ObjectDataProp, UniformData as _,
    VertexAttribute,
};
use gobs_resource::{
    ResourceLifetime, ResourceManager,
    load::{self, AssetType},
};

use crate::{
    Material, MaterialProperties,
    data::{MaterialDataProp, TextureDataProp},
};

#[derive(Debug, Deserialize)]
pub struct MaterialsConfig {
    default: DefaultMaterialConfig,
    materials: HashMap<String, MaterialConfig>,
}

#[derive(Debug, Deserialize)]
struct DefaultMaterialConfig {
    #[serde(default)]
    object_layout: Vec<ObjectDataProp>,
    vertex_attributes: VertexAttribute,
    color_format: ImageFormat,
    depth_format: ImageFormat,
}

#[derive(Debug, Deserialize)]
struct MaterialConfig {
    vertex_shader: String,
    vertex_entry: String,
    fragment_shader: String,
    fragment_entry: String,
    vertex_attributes: Option<VertexAttribute>,
    #[serde(default)]
    blend_mode: BlendMode,
    #[serde(default)]
    cull_mode: CullMode,
    #[serde(default)]
    texture_layout: Vec<TextureDataProp>,
    #[serde(default)]
    material_layout: Vec<MaterialDataProp>,
}

impl MaterialsConfig {
    pub async fn load_resources(filename: &str, resource_manager: &mut ResourceManager) {
        let resources = load::load_string(filename, AssetType::RESOURCES)
            .await
            .unwrap();

        Self::load_resources_with_data(&resources, resource_manager);
    }

    pub fn load_resources_sync(filename: &str, resource_manager: &mut ResourceManager) {
        let resources = load::load_string_sync(filename, AssetType::RESOURCES).unwrap();

        Self::load_resources_with_data(&resources, resource_manager);
    }

    pub fn load_resources_with_data(data: &str, resource_manager: &mut ResourceManager) {
        let config: MaterialsConfig = ron::from_str(data).unwrap();

        config.load_materials(resource_manager);
    }

    fn load_materials(&self, resource_manager: &mut ResourceManager) {
        let mut object_layout = ObjectDataLayout::new(AlignMode::Std140);
        for prop in &self.default.object_layout {
            object_layout = object_layout.prop(*prop);
        }

        for (name, material) in &self.materials {
            let vertex_attributes = match material.vertex_attributes {
                Some(vertex_attributes) => vertex_attributes,
                None => self.default.vertex_attributes,
            };

            let mut props = MaterialProperties::new(
                name,
                &material.vertex_shader,
                &material.vertex_entry,
                &material.fragment_shader,
                &material.fragment_entry,
                vertex_attributes,
                object_layout.clone(),
                self.default.color_format,
                self.default.depth_format,
            )
            .cull_mode(material.cull_mode)
            .blend_mode(material.blend_mode);

            tracing::debug!(target: logger::INIT, "Loading material {} with texture bindings: {:#?}", name, &material.texture_layout);
            tracing::debug!(target: logger::INIT, "Loading material {} with material bindings: {:#?}", name, &material.material_layout);

            for prop in &material.texture_layout {
                props = props.texture(*prop);
            }

            for prop in &material.material_layout {
                props = props.property(*prop);
            }

            resource_manager.add::<Material>(props, ResourceLifetime::Static, true);
        }
    }
}
