use crate::model::Texture;
use log::*;
use uuid::Uuid;

use crate::render::Gfx;

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

    pub async fn diffuse_texture(mut self, file: &str, gfx: &Gfx) -> Self {
        self.diffuse_texture = Some(Texture::load_texture(file, false, gfx).await.unwrap());

        self
    }

    pub async fn normal_texture(mut self, file: &str, gfx: &Gfx) -> Self {
        self.normal_texture = Some(Texture::load_texture(file, true, gfx).await.unwrap());

        self
    }

    pub fn build(self, gfx: &Gfx, layout: &wgpu::BindGroupLayout) -> Material {
        Material::new(
            self.name,
            gfx,
            layout,
            self.diffuse_texture.unwrap(),
            self.normal_texture.unwrap(),
        )
    }
}

pub struct Material {
    pub id: Uuid,
    pub name: String,
    pub diffuse_texture: Texture,
    pub normal_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        name: String,
        gfx: &Gfx,
        layout: &wgpu::BindGroupLayout,
        diffuse_texture: Texture,
        normal_texture: Texture,
    ) -> Self {
        info!("Create Material bind group");
        let bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: None,
        });

        Material {
            id: Uuid::new_v4(),
            name,
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}
