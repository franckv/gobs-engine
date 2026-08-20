use std::collections::HashMap;

use gobs_render_graph::{SceneDataLayout, SceneDataProp};
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
    #[serde(default)]
    scene_layout: Vec<SceneDataProp>,
    vertex_attributes: VertexAttribute,
    color_format: ImageFormat,
    depth_format: ImageFormat,
    #[serde(default)]
    texture_array_size: u32,
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
    texture_indexing: bool,
    #[serde(default)]
    material_layout: Vec<MaterialDataProp>,
    #[serde(default)]
    scene_layout: Vec<SceneDataProp>,
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

        let mut default_scene_layout = SceneDataLayout::new(AlignMode::Std140);
        for prop in &self.default.scene_layout {
            default_scene_layout = default_scene_layout.prop(*prop);
        }

        for (name, material) in &self.materials {
            let vertex_attributes = match material.vertex_attributes {
                Some(vertex_attributes) => vertex_attributes,
                None => self.default.vertex_attributes,
            };

            let scene_layout = if material.scene_layout.is_empty() {
                default_scene_layout.clone()
            } else {
                let mut scene_layout = SceneDataLayout::new(AlignMode::Std140);
                for prop in &material.scene_layout {
                    scene_layout = scene_layout.prop(*prop);
                }
                scene_layout
            };

            let mut props = MaterialProperties::new(
                name,
                &material.vertex_shader,
                &material.vertex_entry,
                &material.fragment_shader,
                &material.fragment_entry,
                vertex_attributes,
                object_layout.clone(),
                scene_layout,
                self.default.color_format,
                self.default.depth_format,
            )
            .cull_mode(material.cull_mode)
            .blend_mode(material.blend_mode);

            tracing::debug!(target: logger::INIT, "Loading material {} with texture bindings: {:#?}", name, &material.texture_layout);
            tracing::debug!(target: logger::INIT, "Loading material {} with material bindings: {:#?}", name, &material.material_layout);

            for prop in &material.material_layout {
                props = props.property(*prop);
            }

            props = props.textures(
                &material.texture_layout,
                material.texture_indexing,
                self.default.texture_array_size,
            );

            resource_manager.add::<Material>(props, ResourceLifetime::Static, true);
        }
    }
}
