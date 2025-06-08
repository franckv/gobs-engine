use std::sync::Arc;

use gobs_render::{
    Material, MaterialInstance, MaterialProperties, MaterialProperty, Texture, TextureProperties,
};
use gobs_render_graph::{BlendMode, GfxContext, RenderPass};
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
    pub texture: ResourceHandle<Material>,
    pub transparent_texture: ResourceHandle<Material>,
    pub texture_normal: ResourceHandle<Material>,
    pub transparent_texture_normal: ResourceHandle<Material>,
    pub color_instance: Arc<MaterialInstance>,
    pub transparent_color_instance: Arc<MaterialInstance>,
}

impl MaterialManager {
    pub fn new(
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
    ) -> Result<Self, AssetError> {
        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        let texture = resource_manager.add(
            MaterialProperties::new(
                ctx,
                "gltf.texture.vert.spv",
                "main",
                "gltf.texture.frag.spv",
                "main",
                vertex_attributes,
                pass.clone(),
            )
            .prop("diffuse", MaterialProperty::Texture),
            ResourceLifetime::Static,
        );

        let transparent_texture = resource_manager.add(
            MaterialProperties::new(
                ctx,
                "gltf.texture.vert.spv",
                "main",
                "gltf.texture.frag.spv",
                "main",
                vertex_attributes,
                pass.clone(),
            )
            .prop("diffuse", MaterialProperty::Texture)
            .blend_mode(BlendMode::Alpha),
            ResourceLifetime::Static,
        );

        let texture_normal = resource_manager.add(
            MaterialProperties::new(
                ctx,
                "gltf.texture.vert.spv",
                "main",
                "gltf.texture_n.frag.spv",
                "main",
                vertex_attributes,
                pass.clone(),
            )
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture),
            ResourceLifetime::Static,
        );

        let transparent_texture_normal = resource_manager.add(
            MaterialProperties::new(
                ctx,
                "gltf.texture.vert.spv",
                "main",
                "gltf.texture_n.frag.spv",
                "main",
                vertex_attributes,
                pass.clone(),
            )
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .blend_mode(BlendMode::Alpha),
            ResourceLifetime::Static,
        );

        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::COLOR
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        let color = resource_manager.add(
            MaterialProperties::new(
                ctx,
                "gltf.color_light.vert.spv",
                "main",
                "gltf.color_light.frag.spv",
                "main",
                vertex_attributes,
                pass.clone(),
            ),
            ResourceLifetime::Static,
        );

        let transparent_color = resource_manager.add(
            MaterialProperties::new(
                ctx,
                "gltf.color_light.vert.spv",
                "main",
                "gltf.color_light.frag.spv",
                "main",
                vertex_attributes,
                pass,
            )
            .blend_mode(BlendMode::Alpha),
            ResourceLifetime::Static,
        );

        let texture_manager = TextureManager::new(resource_manager);

        let default_material_instance =
            MaterialInstance::new(texture, vec![texture_manager.default_texture]);
        tracing::debug!(target: "resources", "Default material id: {}", default_material_instance.id);

        let color_instance = MaterialInstance::new(color, vec![]);
        tracing::debug!(target: "resources", "Color material id: {}", color_instance.id);

        let transparent_color_instance = MaterialInstance::new(transparent_color, vec![]);
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
            BlendMode::Alpha => MaterialInstance::new(self.transparent_texture, vec![texture]),
            _ => MaterialInstance::new(self.texture, vec![texture]),
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
            BlendMode::Alpha => {
                MaterialInstance::new(self.transparent_texture_normal, vec![diffuse, normal])
            }
            _ => MaterialInstance::new(self.texture_normal, vec![diffuse, normal]),
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
