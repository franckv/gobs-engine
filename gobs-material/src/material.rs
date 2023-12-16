use std::sync::{Arc, RwLock};

use uuid::Uuid;

use gobs_core as core;

use core::Color;

use crate::texture::{Texture, TextureType};

pub type MaterialId = Uuid;

pub struct Material {
    pub id: MaterialId,
    pub name: String,
    pub diffuse_texture: RwLock<Texture>,
    pub normal_texture: RwLock<Texture>,
}

impl Material {
    pub fn new(name: String, diffuse_texture: Texture, normal_texture: Texture) -> Arc<Self> {
        log::debug!("Create Material bind group");

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

    pub async fn diffuse_buffer(mut self, buffer: &[Color], width: u32, height: u32) -> Self {
        self.diffuse_texture = Some(Texture::new(
            "framebuffer",
            TextureType::IMAGE,
            buffer
                .iter()
                .flat_map(|c| Into::<[u8; 4]>::into(*c))
                .collect::<Vec<u8>>(),
            width,
            height,
        ));

        self
    }

    pub async fn diffuse_color(mut self, color: Color) -> Self {
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
            None => Texture::from_color(Color::WHITE, TextureType::IMAGE),
        };

        let normal_texture = match self.normal_texture {
            Some(normal_texture) => normal_texture,
            None => Texture::from_color(Color::BLACK, TextureType::NORMAL),
        };

        Material::new(self.name, diffuse_texture, normal_texture)
    }
}
