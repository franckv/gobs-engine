use std::collections::HashMap;

use serde::Deserialize;

use gobs_gfx::BlendMode;
use gobs_render_low::{GfxContext, ObjectDataLayout, ObjectDataProp, TextureDataProp};
use gobs_resource::{
    geometry::VertexAttribute,
    load::{self, AssetType},
    manager::ResourceManager,
    resource::ResourceLifetime,
};

use crate::{Material, MaterialProperties};

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
    texture_layout: Vec<TextureDataProp>,
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

        config.load_materials(ctx, resource_manager);
    }

    pub fn load_resources_sync(
        ctx: &GfxContext,
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) {
        let resources = load::load_string_sync(filename, AssetType::RESOURCES).unwrap();
        let config: MaterialsConfig = ron::from_str(&resources).unwrap();

        config.load_materials(ctx, resource_manager);
    }

    fn load_materials(&self, ctx: &GfxContext, resource_manager: &mut ResourceManager) {
        let mut object_layout = ObjectDataLayout::builder();
        for prop in &self.default.object_layout {
            object_layout = object_layout.prop(*prop);
        }
        let object_layout = object_layout.build();

        for (name, material) in &self.materials {
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

            for prop in &material.texture_layout {
                props = props.texture(*prop);
            }

            resource_manager.add::<Material>(props, ResourceLifetime::Static);
        }
    }
}
