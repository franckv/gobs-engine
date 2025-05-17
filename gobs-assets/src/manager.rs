use std::sync::Arc;

use gobs_render::{
    BlendMode, GfxContext, Material, MaterialInstance, MaterialProperty, RenderPass, Texture,
    TextureProperties,
};
use gobs_resource::{
    geometry::VertexAttribute,
    manager::ResourceManager,
    resource::{ResourceHandle, ResourceLifetime},
};

use crate::AssetError;

pub struct TextureManager {
    pub textures: Vec<ResourceHandle<Texture>>,
    pub default_texture: ResourceHandle<Texture>,
}

impl TextureManager {
    pub fn new(resource_manager: &mut ResourceManager) -> Self {
        let default_texture =
            resource_manager.add(TextureProperties::default(), ResourceLifetime::Static);

        TextureManager {
            textures: vec![],
            default_texture,
        }
    }

    pub fn add(&mut self, texture: ResourceHandle<Texture>) {
        self.textures.push(texture);
    }

    pub fn add_default(&mut self) {
        self.textures.push(self.default_texture);
    }
}

pub struct MaterialManager {
    pub texture_manager: TextureManager,
    pub instances: Vec<Arc<MaterialInstance>>,
    pub default_material_instance: Arc<MaterialInstance>,
    pub texture: Arc<Material>,
    pub transparent_texture: Arc<Material>,
    pub texture_normal: Arc<Material>,
    pub transparent_texture_normal: Arc<Material>,
    pub color_instance: Arc<MaterialInstance>,
    pub transparent_color_instance: Arc<MaterialInstance>,
}

impl MaterialManager {
    pub fn new(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
    ) -> Result<Self, AssetError> {
        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        let texture = Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture.frag.spv")?
            .vertex_attributes(vertex_attributes)
            .prop("diffuse", MaterialProperty::Texture)
            .build(pass.clone());

        let transparent_texture =
            Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture.frag.spv")?
                .vertex_attributes(vertex_attributes)
                .prop("diffuse", MaterialProperty::Texture)
                .blend_mode(BlendMode::Alpha)
                .build(pass.clone());

        let texture_normal =
            Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture_n.frag.spv")?
                .vertex_attributes(vertex_attributes)
                .prop("diffuse", MaterialProperty::Texture)
                .prop("normal", MaterialProperty::Texture)
                .build(pass.clone());

        let transparent_texture_normal =
            Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture_n.frag.spv")?
                .vertex_attributes(vertex_attributes)
                .prop("diffuse", MaterialProperty::Texture)
                .prop("normal", MaterialProperty::Texture)
                .blend_mode(BlendMode::Alpha)
                .build(pass.clone());

        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::COLOR
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        let color = Material::builder(
            ctx,
            "gltf.color_light.vert.spv",
            "gltf.color_light.frag.spv",
        )?
        .vertex_attributes(vertex_attributes)
        .build(pass.clone());

        let transparent_color = Material::builder(
            ctx,
            "gltf.color_light.vert.spv",
            "gltf.color_light.frag.spv",
        )?
        .vertex_attributes(vertex_attributes)
        .blend_mode(BlendMode::Alpha)
        .build(pass.clone());

        let texture_manager = TextureManager::new(resource_manager);

        let default_material_instance = texture
            .clone()
            .instantiate(vec![texture_manager.default_texture]);
        tracing::debug!(target: "resources", "Default material id: {}", default_material_instance.id);

        let color_instance = color.instantiate(vec![]);
        tracing::debug!(target: "resources", "Color material id: {}", color_instance.id);

        let transparent_color_instance = transparent_color.instantiate(vec![]);
        tracing::debug!(target: "resources", "Color material id: {}", transparent_color_instance.id);

        Ok(MaterialManager {
            texture_manager,
            instances: vec![],
            default_material_instance,
            texture,
            transparent_texture,
            texture_normal,
            transparent_texture_normal,
            color_instance,
            transparent_color_instance,
        })
    }

    pub fn add_texture(&mut self, texture: ResourceHandle<Texture>) {
        self.texture_manager.add(texture);
    }

    pub fn add_default_texture(&mut self) {
        self.texture_manager.add_default();
    }

    pub fn add_texture_instance(
        &mut self,
        alpha: BlendMode,
        texture: usize,
    ) -> Arc<MaterialInstance> {
        let texture = self.texture_manager.textures[texture];

        let material_instance = match alpha {
            BlendMode::Alpha => self.transparent_texture.instantiate(vec![texture]),
            _ => self.texture.instantiate(vec![texture]),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }

    pub fn add_texture_normal_instance(
        &mut self,
        alpha: BlendMode,
        diffuse: usize,
        normal: usize,
    ) -> Arc<MaterialInstance> {
        let diffuse = self.texture_manager.textures[diffuse];
        let normal = self.texture_manager.textures[normal];

        let material_instance = match alpha {
            BlendMode::Alpha => self
                .transparent_texture_normal
                .instantiate(vec![diffuse, normal]),
            _ => self.texture_normal.instantiate(vec![diffuse, normal]),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }

    pub fn add_color_instance(&mut self, alpha: BlendMode) -> Arc<MaterialInstance> {
        let material_instance = match alpha {
            BlendMode::Alpha => self.transparent_color_instance.clone(),
            _ => self.color_instance.clone(),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }
}
