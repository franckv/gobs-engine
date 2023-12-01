use std::sync::{Arc, RwLock};

use log::*;
use uuid::Uuid;

use gobs_core as core;

use core::material::texture::{Texture, TextureType};

pub type MaterialId = Uuid;

pub struct Material {
    pub id: MaterialId,
    pub name: String,
    pub diffuse_texture: RwLock<Texture>,
    pub normal_texture: RwLock<Texture>,
}

impl Material {
    pub fn new(name: String, diffuse_texture: Texture, normal_texture: Texture) -> Arc<Self> {
        info!("Create Material bind group");

        Arc::new(Material {
            id: Uuid::new_v4(),
            name,
            diffuse_texture: RwLock::new(diffuse_texture),
            normal_texture: RwLock::new(normal_texture),
        })
    }
}

pub struct MaterialBuilder {
    name: String,
    diffuse_texture: Option<Texture>,
    normal_texture: Option<Texture>,
}

impl MaterialBuilder {
    pub fn new(name: &str) -> Self {
        MaterialBuilder {
            name: name.to_string(),
            diffuse_texture: None,
            normal_texture: None,
        }
    }

    pub async fn diffuse_buffer(mut self, buffer: &[u8], width: u32, height: u32) -> Self {
        self.diffuse_texture = Some(Texture::new(
            "framebuffer",
            TextureType::IMAGE,
            buffer,
            width,
            height,
        ));

        self
    }

    pub async fn diffuse_color(mut self, color: [u8; 4]) -> Self {
        self.diffuse_texture = Some(Texture::from_color(color, TextureType::IMAGE));

        self
    }

    pub async fn diffuse_texture_t(mut self, texture: Texture) -> Self {
        self.diffuse_texture = Some(texture);

        self
    }

    pub async fn diffuse_texture(mut self, file: &str) -> Self {
        self.diffuse_texture = Some(Texture::from_file(file, TextureType::IMAGE).await.unwrap());

        self
    }

    pub async fn normal_texture(mut self, file: &str) -> Self {
        self.normal_texture = Some(Texture::from_file(file, TextureType::NORMAL).await.unwrap());

        self
    }

    pub fn build(self) -> Arc<Material> {
        let diffuse_texture = match self.diffuse_texture {
            Some(diffuse_texture) => diffuse_texture,
            None => Texture::from_color([255, 255, 255, 1], TextureType::IMAGE),
        };

        let normal_texture = match self.normal_texture {
            Some(normal_texture) => normal_texture,
            None => Texture::from_color([0, 0, 0, 1], TextureType::NORMAL),
        };

        Material::new(self.name, diffuse_texture, normal_texture)
    }
}
