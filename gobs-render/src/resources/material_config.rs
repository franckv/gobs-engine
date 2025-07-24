use std::collections::HashMap;

use serde::Deserialize;

use gobs_gfx::BlendMode;
use gobs_render_low::{GfxContext, ObjectDataLayout, ObjectDataProp};
use gobs_resource::{
    geometry::VertexAttribute,
    load::{self, AssetType},
    manager::ResourceManager,
    resource::ResourceLifetime,
};

use crate::{Material, MaterialProperties, MaterialProperty};

#[derive(Debug, Deserialize)]
pub struct MaterialsConfig {
    default: DefaultMaterialConfig,
    materials: HashMap<String, MaterialConfig>,
}

#[derive(Debug, Deserialize)]
struct DefaultMaterialConfig {
    #[serde(default)]
    object_layout: Vec<ObjectDataProp>,
}

#[derive(Debug, Deserialize)]
struct MaterialConfig {
    vertex_shader: String,
    vertex_entry: String,
    fragment_shader: String,
    fragment_entry: String,
    vertex_attributes: VertexAttribute,
    #[serde(default)]
    blend_mode: BlendMode,
    #[serde(default)]
    properties: HashMap<String, MaterialProperty>,
}

impl MaterialsConfig {
    pub async fn load_resources(
        ctx: &GfxContext,
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) {
        let resources = load::load_string(filename, AssetType::RESOURCES)
            .await
            .unwrap();
        let config: MaterialsConfig = ron::from_str(&resources).unwrap();

        let mut object_layout = ObjectDataLayout::builder();
        for prop in &config.default.object_layout {
            object_layout = object_layout.prop(*prop);
        }
        let object_layout = object_layout.build();

        for (name, material) in &config.materials {
            let mut props = MaterialProperties::new(
                ctx,
                name,
                &material.vertex_shader,
                &material.vertex_entry,
                &material.fragment_shader,
                &material.fragment_entry,
                material.vertex_attributes,
                &object_layout,
            )
            .blend_mode(material.blend_mode);

            for (prop_name, prop_type) in &material.properties {
                props = props.prop(prop_name, *prop_type);
            }

            resource_manager.add::<Material>(props, ResourceLifetime::Static);
        }
    }
}
