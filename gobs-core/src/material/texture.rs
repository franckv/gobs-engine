use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer};
use uuid::Uuid;

use gobs_utils as utils;

use crate::Color;
use utils::load::{self, AssetType};

#[derive(Copy, Clone, Debug)]
pub enum TextureFormat {
    Rgba8UnormSrgb,
    Rgba8Unorm,
    Depth32Float,
}

impl TextureFormat {
    pub fn size(&self) -> u32 {
        match self {
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Rgba8Unorm => 4,
            TextureFormat::Depth32Float => 4,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TextureType {
    IMAGE,
    NORMAL,
    DEPTH,
}

impl TextureType {
    pub fn format(&self) -> TextureFormat {
        match self {
            TextureType::IMAGE => TextureFormat::Rgba8UnormSrgb,
            TextureType::NORMAL => TextureFormat::Rgba8Unorm,
            TextureType::DEPTH => TextureFormat::Depth32Float,
        }
    }
}

pub type TextureId = Uuid;

#[derive(Clone)]
pub struct Texture {
    pub id: TextureId,
    pub name: String,
    pub ty: TextureType,
    pub format: TextureFormat,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub dirty: bool,
}

impl Texture {
    pub fn new(name: &str, ty: TextureType, data: &[u8], width: u32, height: u32) -> Self {
        Texture {
            id: Uuid::new_v4(),
            name: name.to_string(),
            ty,
            format: ty.format(),
            data: data.to_owned(),
            width,
            height,
            dirty: false,
        }
    }

    pub async fn from_file(file_name: &str, ty: TextureType) -> Result<Self> {
        let img = load::load_image(file_name, AssetType::IMAGE).await?;

        Ok(Self::new(
            file_name,
            ty,
            &img.to_rgba8(),
            img.dimensions().0,
            img.dimensions().1,
        ))
    }

    pub fn from_color(c: Color, ty: TextureType) -> Self {
        let img = ImageBuffer::from_pixel(1, 1, c.into());

        let img_color = DynamicImage::ImageRgba8(img);

        Self::new(
            "Color texture",
            ty,
            &img_color.to_rgba8(),
            img_color.dimensions().0,
            img_color.dimensions().1,
        )
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn patch_texture(
        &mut self,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        let mut new_data: Vec<u8> = self.data.clone();

        for x in 0..width {
            for y in 0..height {
                let local_idx = (x + y * width) as usize;
                let global_idx = ((start_x + x) + (start_y + y) * self.width) as usize;

                new_data[4 * global_idx] = data[4 * local_idx];
                new_data[4 * global_idx + 1] = data[4 * local_idx + 1];
                new_data[4 * global_idx + 2] = data[4 * local_idx + 2];
                new_data[4 * global_idx + 3] = data[4 * local_idx + 3];
            }
        }

        self.data = new_data;
        self.dirty = true;
    }
}
