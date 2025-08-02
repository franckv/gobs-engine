use gobs_core::logger;
use gobs_render::{
    BlendMode, GfxContext, Material, MaterialInstance, MaterialInstanceProperties, MaterialsConfig,
    Texture, TextureProperties,
};
use gobs_resource::{
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
    pub instances: Vec<ResourceHandle<MaterialInstance>>,
    pub default_material_instance: ResourceHandle<MaterialInstance>,
    pub texture: ResourceHandle<Material>,
    pub transparent_texture: ResourceHandle<Material>,
    pub texture_normal: ResourceHandle<Material>,
    pub transparent_texture_normal: ResourceHandle<Material>,
    pub color_instance: ResourceHandle<MaterialInstance>,
    pub transparent_color_instance: ResourceHandle<MaterialInstance>,
}

impl MaterialManager {
    pub fn new(
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, AssetError> {
        MaterialsConfig::load_resources_sync(ctx, "gltf_materials.ron", resource_manager);

        let texture = resource_manager
            .get_by_name("gltf.texture")
            .ok_or(AssetError::AssetNotFound)?;
        let transparent_texture = resource_manager
            .get_by_name("gltf.texture.transparent")
            .ok_or(AssetError::AssetNotFound)?;
        let texture_normal = resource_manager
            .get_by_name("gltf.texture.normal")
            .ok_or(AssetError::AssetNotFound)?;
        let transparent_texture_normal = resource_manager
            .get_by_name("gltf.texture.transparent.normal")
            .ok_or(AssetError::AssetNotFound)?;
        let color = resource_manager
            .get_by_name("gltf.color")
            .ok_or(AssetError::AssetNotFound)?;
        let transparent_color = resource_manager
            .get_by_name("gltf.color.transparent")
            .ok_or(AssetError::AssetNotFound)?;

        let texture_manager = TextureManager::new(resource_manager);

        let default_material_instance = resource_manager.add::<MaterialInstance>(
            MaterialInstanceProperties::new(
                "default",
                texture,
                vec![texture_manager.default_texture],
            ),
            ResourceLifetime::Static,
        );

        tracing::debug!(target: logger::RESOURCES, "Default material id: {}", default_material_instance.id);

        let color_instance = resource_manager.add::<MaterialInstance>(
            MaterialInstanceProperties::new("color", color, vec![]),
            ResourceLifetime::Static,
        );
        tracing::debug!(target: logger::RESOURCES, "Color material id: {}", color_instance.id);

        let transparent_color_instance = resource_manager.add::<MaterialInstance>(
            MaterialInstanceProperties::new("transparent color", transparent_color, vec![]),
            ResourceLifetime::Static,
        );
        tracing::debug!(target: logger::RESOURCES, "Color material id: {}", transparent_color_instance.id);

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
        name: &str,
        resource_manager: &mut ResourceManager,
        alpha: BlendMode,
        texture: usize,
    ) -> ResourceHandle<MaterialInstance> {
        let texture = self.texture_manager.textures[texture];

        let material_instance = match alpha {
            BlendMode::Alpha => resource_manager.add::<MaterialInstance>(
                MaterialInstanceProperties::new(name, self.transparent_texture, vec![texture]),
                ResourceLifetime::Static,
            ),
            _ => resource_manager.add::<MaterialInstance>(
                MaterialInstanceProperties::new(name, self.texture, vec![texture]),
                ResourceLifetime::Static,
            ),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }

    pub fn add_texture_normal_instance(
        &mut self,
        name: &str,
        resource_manager: &mut ResourceManager,
        alpha: BlendMode,
        diffuse: usize,
        normal: usize,
    ) -> ResourceHandle<MaterialInstance> {
        let diffuse = self.texture_manager.textures[diffuse];
        let normal = self.texture_manager.textures[normal];

        let material_instance = match alpha {
            BlendMode::Alpha => resource_manager.add::<MaterialInstance>(
                MaterialInstanceProperties::new(
                    name,
                    self.transparent_texture_normal,
                    vec![diffuse, normal],
                ),
                ResourceLifetime::Static,
            ),
            _ => resource_manager.add::<MaterialInstance>(
                MaterialInstanceProperties::new(name, self.texture_normal, vec![diffuse, normal]),
                ResourceLifetime::Static,
            ),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }

    pub fn add_color_instance(&mut self, alpha: BlendMode) -> ResourceHandle<MaterialInstance> {
        let material_instance = match alpha {
            BlendMode::Alpha => self.transparent_color_instance.clone(),
            _ => self.color_instance.clone(),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }
}
