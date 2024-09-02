use std::sync::Arc;

use gobs_render::{BlendMode, Context, Material, MaterialInstance, MaterialProperty, RenderPass};
use gobs_resource::{geometry::VertexFlag, material::Texture};

pub struct TextureManager {
    pub textures: Vec<Arc<Texture>>,
    pub default_texture: Arc<Texture>,
}

impl TextureManager {
    pub fn new() -> Self {
        let default_texture = Texture::default();

        TextureManager {
            textures: vec![],
            default_texture,
        }
    }

    pub fn add(&mut self, texture: Arc<Texture>) {
        self.textures.push(texture);
    }

    pub fn add_default(&mut self) {
        self.textures.push(self.default_texture.clone());
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
    pub fn new(ctx: &Context, pass: RenderPass) -> Self {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        let texture = Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .build(pass.clone());

        let transparent_texture =
            Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture.frag.spv")
                .vertex_flags(vertex_flags)
                .prop("diffuse", MaterialProperty::Texture)
                .blend_mode(BlendMode::Alpha)
                .build(pass.clone());

        let texture_normal =
            Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture_n.frag.spv")
                .vertex_flags(vertex_flags)
                .prop("diffuse", MaterialProperty::Texture)
                .prop("normal", MaterialProperty::Texture)
                .build(pass.clone());

        let transparent_texture_normal =
            Material::builder(ctx, "gltf.texture.vert.spv", "gltf.texture_n.frag.spv")
                .vertex_flags(vertex_flags)
                .prop("diffuse", MaterialProperty::Texture)
                .prop("normal", MaterialProperty::Texture)
                .blend_mode(BlendMode::Alpha)
                .build(pass.clone());

        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::COLOR
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        let color = Material::builder(
            ctx,
            "gltf.color_light.vert.spv",
            "gltf.color_light.frag.spv",
        )
        .vertex_flags(vertex_flags)
        .build(pass.clone());

        let transparent_color = Material::builder(
            ctx,
            "gltf.color_light.vert.spv",
            "gltf.color_light.frag.spv",
        )
        .vertex_flags(vertex_flags)
        .blend_mode(BlendMode::Alpha)
        .build(pass.clone());

        let texture_manager = TextureManager::new();

        let default_material_instance = texture
            .clone()
            .instantiate(vec![texture_manager.default_texture.clone()]);
        tracing::debug!("Default material id: {}", default_material_instance.id);

        let color_instance = color.instantiate(vec![]);
        tracing::debug!("Color material id: {}", color_instance.id);

        let transparent_color_instance = transparent_color.instantiate(vec![]);
        tracing::debug!("Color material id: {}", transparent_color_instance.id);

        MaterialManager {
            texture_manager,
            instances: vec![],
            default_material_instance,
            texture,
            transparent_texture,
            texture_normal,
            transparent_texture_normal,
            color_instance,
            transparent_color_instance,
        }
    }

    pub fn add_texture(&mut self, texture: Arc<Texture>) {
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
        let texture = self.texture_manager.textures[texture].clone();

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
        let diffuse = self.texture_manager.textures[diffuse].clone();
        let normal = self.texture_manager.textures[normal].clone();

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
