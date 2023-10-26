use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer};

use gobs_utils as utils;

use crate::render::Gfx;
use utils::load::{self, AssetType};

pub enum TextureType {
    IMAGE,
    NORMAL,
    DEPTH,
}

impl TextureType {
    pub fn format(&self) -> wgpu::TextureFormat {
        match self {
            TextureType::IMAGE => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureType::NORMAL => wgpu::TextureFormat::Rgba8Unorm,
            TextureType::DEPTH => wgpu::TextureFormat::Depth32Float,
        }
    }

    pub fn usage(&self) -> wgpu::TextureUsages {
        match self {
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

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub cols: u32,
    pub rows: u32,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(
        gfx: &Gfx,
        label: &str,
        ty: TextureType,
        width: u32,
        height: u32,
        cols: u32,
        rows: u32,
        img: Option<&image::DynamicImage>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ty.format(),
            usage: ty.usage(),
            view_formats: &[],
        };

        let texture = gfx.device().create_texture(&desc);

        if let Some(img) = img {
            let rgba = img.to_rgba8();
            gfx.queue().write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: match ty {
                TextureType::IMAGE => wgpu::FilterMode::Nearest,
                TextureType::NORMAL => wgpu::FilterMode::Nearest,
                TextureType::DEPTH => wgpu::FilterMode::Linear,
            },
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: match ty {
                TextureType::DEPTH => Some(wgpu::CompareFunction::LessEqual),
                _ => Default::default(),
            },
            lod_min_clamp: 0.0,
            lod_max_clamp: match ty {
                TextureType::DEPTH => 100.0,
                _ => 32.,
            },
            ..Default::default()
        });

        Self {
            width: gfx.width(),
            height: gfx.height(),
            cols,
            rows,
            texture,
            view,
            sampler,
        }
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub async fn load_texture(
        gfx: &Gfx,
        file_name: &str,
        cols: u32,
        rows: u32,
        is_normal_map: bool,
    ) -> Result<Self> {
        let img = load::load_image(file_name, AssetType::IMAGE).await?;

        let ty = match is_normal_map {
            true => TextureType::NORMAL,
            false => TextureType::IMAGE,
        };

        Ok(Self::new(
            gfx,
            file_name,
            ty,
            img.dimensions().0,
            img.dimensions().1,
            cols,
            rows,
            Some(&img),
        ))
    }

    pub fn from_color(gfx: &Gfx, c: [u8; 4], is_normal_map: bool) -> Self {
        let pixel = image::Rgba([c[0], c[1], c[2], c[3]]);

        let img = ImageBuffer::from_pixel(1, 1, pixel);

        let img_color = DynamicImage::ImageRgba8(img);

        let ty = match is_normal_map {
            true => TextureType::NORMAL,
            false => TextureType::IMAGE,
        };

        Self::new(
            gfx,
            "Color texture",
            ty,
            img_color.dimensions().0,
            img_color.dimensions().1,
            1,
            1,
            Some(&img_color),
        )
    }
}
