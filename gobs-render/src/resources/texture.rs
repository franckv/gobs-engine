use gobs_material as material;

use material::{Texture, TextureFormat, TextureType};

use crate::context::Gfx;

pub struct TextureBuffer {
    pub texture: Texture,
    pub texture_buffer: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureBuffer {
    pub fn new(gfx: &Gfx, texture: Texture) -> Self {
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(&texture.name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::format(texture.format),
            usage: Self::usage(texture.ty),
            view_formats: &[],
        };

        let texture_buffer = gfx.device().create_texture(&desc);

        if !texture.data().is_empty() {
            gfx.queue().write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture_buffer,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &texture.data(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * texture.width),
                    rows_per_image: Some(texture.height),
                },
                size,
            );
        }

        let view = texture_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: match texture.ty {
                TextureType::IMAGE => wgpu::FilterMode::Nearest,
                TextureType::NORMAL => wgpu::FilterMode::Nearest,
                TextureType::DEPTH => wgpu::FilterMode::Linear,
            },
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: match texture.ty {
                TextureType::DEPTH => Some(wgpu::CompareFunction::LessEqual),
                _ => Default::default(),
            },
            lod_min_clamp: 0.,
            lod_max_clamp: match texture.ty {
                TextureType::DEPTH => 100.,
                _ => 32.,
            },
            ..Default::default()
        });

        Self {
            texture,
            texture_buffer,
            view,
            sampler,
        }
    }

    // TODO: Texture dimensions (width/height) cannot be updated immutably
    pub fn patch_texture(
        &self,
        gfx: &Gfx,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        img: &[u8],
    ) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let origin = wgpu::Origin3d {
            x: start_x,
            y: start_y,
            z: 0,
        };

        gfx.queue().write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture_buffer,
                mip_level: 0,
                origin,
            },
            img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.texture.format.size() * width),
                rows_per_image: Some(height),
            },
            size,
        );
    }

    pub fn format(format: TextureFormat) -> wgpu::TextureFormat {
        match format {
            TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        }
    }

    pub fn usage(ty: TextureType) -> wgpu::TextureUsages {
        match ty {
            TextureType::IMAGE => {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
            }
            TextureType::NORMAL => {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
            }
            TextureType::DEPTH => {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
            }
        }
    }
}
