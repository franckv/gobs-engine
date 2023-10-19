use crate::model::Texture;
use log::*;

use crate::render::Gfx;

pub struct Material {
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
            name,
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}
