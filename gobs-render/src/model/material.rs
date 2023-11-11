use std::sync::Arc;

use crate::model::Texture;
use log::*;

use crate::context::Gfx;

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

    pub async fn diffuse_color(mut self, gfx: &Gfx, color: [u8; 4]) -> Self {
        self.diffuse_texture = Some(Texture::from_color(gfx, color, false));

        self
    }

    pub async fn diffuse_texture_t(mut self, texture: Texture) -> Self {
        self.diffuse_texture = Some(texture);

        self
    }

    pub async fn diffuse_texture(mut self, gfx: &Gfx, file: &str) -> Self {
        self.diffuse_texture = Some(Texture::load_texture(gfx, file, false).await.unwrap());

        self
    }

    pub async fn normal_texture(mut self, gfx: &Gfx, file: &str) -> Self {
        self.normal_texture = Some(Texture::load_texture(gfx, file, true).await.unwrap());

        self
    }

    pub fn build(self, gfx: &Gfx) -> Arc<Material> {
        let diffuse_texture = match self.diffuse_texture {
            Some(diffuse_texture) => diffuse_texture,
            None => Texture::from_color(gfx, [255, 255, 255, 1], false),
        };

        let normal_texture = match self.normal_texture {
            Some(normal_texture) => normal_texture,
            None => Texture::from_color(gfx, [0, 0, 0, 1], true),
        };

        Material::new(self.name, diffuse_texture, normal_texture)
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub normal_texture: Texture,
}

impl Material {
    pub fn new(name: String, diffuse_texture: Texture, normal_texture: Texture) -> Arc<Self> {
        info!("Create Material bind group");

        Arc::new(Material {
            name,
            diffuse_texture,
            normal_texture,
        })
    }

    pub fn bind_group(&self, gfx: &Gfx, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.normal_texture.sampler),
                },
            ],
            label: None,
        })
    }
}
