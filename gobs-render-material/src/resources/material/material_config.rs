use std::collections::HashMap;

use serde::Deserialize;

use gobs_core::{ConfigReader as _, GobsConfig, ImageFormat, logger};
use gobs_graphics::VertexAttribute;
use gobs_render_hal::{
    BlendMode, CullMode, ObjectDataLayout, ObjectDataProp, RenderHalConfig, UniformData as _,
};
use gobs_resource::{
    ResourceLifetime, ResourceManager,
    load::{self, AssetType},
};

use crate::{
    Material, MaterialProperties,
    data::{MaterialDataProp, SceneDataLayout, SceneDataProp, TextureDataProp},
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
    object_instancing: bool,
    #[serde(default)]
    scene_layout: Vec<SceneDataProp>,
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
    texture_indexing: bool,
    #[serde(default)]
    material_indexing: bool,
    #[serde(default)]
    material_layout: Vec<MaterialDataProp>,
    #[serde(default)]
    scene_layout: Vec<SceneDataProp>,
    #[serde(default)]
    object_layout: Vec<ObjectDataProp>,
    #[serde(default)]
    object_instancing: bool,
}

impl MaterialsConfig {
    pub async fn load_resources(
        config: GobsConfig,
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) {
        let resources = load::load_string(filename, AssetType::RESOURCES)
            .await
            .unwrap();

        Self::load_resources_with_data(config, &resources, resource_manager);
    }

    pub fn load_resources_sync(
        config: GobsConfig,
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) {
        let resources = load::load_string_sync(filename, AssetType::RESOURCES).unwrap();

        Self::load_resources_with_data(config, &resources, resource_manager);
    }

    pub fn load_resources_with_data(
        config: GobsConfig,
        data: &str,
        resource_manager: &mut ResourceManager,
    ) {
        let material_config: MaterialsConfig = ron::from_str(data).unwrap();

        material_config.load_materials(config, resource_manager);
    }

    fn load_materials(&self, config: GobsConfig, resource_manager: &mut ResourceManager) {
        let mut default_object_layout = ObjectDataLayout::new(self.default.object_instancing);
        for prop in &self.default.object_layout {
            default_object_layout = default_object_layout.prop(*prop);
        }

        let mut default_scene_layout = SceneDataLayout::new();
        for prop in &self.default.scene_layout {
            default_scene_layout = default_scene_layout.prop(*prop);
        }

        for (name, material) in &self.materials {
            let vertex_attributes = match material.vertex_attributes {
                Some(vertex_attributes) => vertex_attributes,
                None => self.default.vertex_attributes,
            };

            let object_layout = if material.object_layout.is_empty() {
                default_object_layout.clone()
            } else {
                let mut object_layout = ObjectDataLayout::new(material.object_instancing);
                for prop in &material.object_layout {
                    object_layout = object_layout.prop(*prop);
                }
                object_layout
            };

            let push_layout = if object_layout.instancing {
                ObjectDataLayout::new(false).prop(ObjectDataProp::InstanceBufferAddress)
            } else {
                object_layout.clone()
            };

            let scene_layout = if material.scene_layout.is_empty() {
                default_scene_layout.clone()
            } else {
                let mut scene_layout = SceneDataLayout::new();
                for prop in &material.scene_layout {
                    scene_layout = scene_layout.prop(*prop);
                }
                scene_layout
            };

            let texture_indexing = material.texture_indexing && !material.texture_layout.is_empty();

            let material_indexing =
                material.material_indexing && !material.material_layout.is_empty();

            let mut props = MaterialProperties::new(
                name,
                &material.vertex_shader,
                &material.vertex_entry,
                &material.fragment_shader,
                &material.fragment_entry,
                vertex_attributes,
                object_layout,
                push_layout,
                scene_layout,
                self.default.color_format,
                self.default.depth_format,
                texture_indexing,
                material_indexing,
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
                config.get_int(RenderHalConfig::TextureArraySize),
            );

            resource_manager.add::<Material>(props, ResourceLifetime::Static, true);
        }
    }
}
