use std::sync::Arc;

use anyhow::Result;
use futures::future::try_join_all;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use uuid::Uuid;

use gobs_core::{Color, ImageExtent2D, ImageFormat, SamplerFilter};

use crate::load::{self, AssetType};

pub type TextureId = Uuid;

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Diffuse,
    Normal,
}

impl From<TextureType> for ImageFormat {
    fn from(val: TextureType) -> Self {
        match val {
            TextureType::Diffuse => ImageFormat::R8g8b8a8Srgb,
            TextureType::Normal => ImageFormat::R8g8b8a8Unorm,
        }
    }
}

pub struct Texture {
    pub id: TextureId,
    pub ty: TextureType,
    pub name: String,
    pub format: ImageFormat,
    pub extent: ImageExtent2D,
    pub mag_filter: SamplerFilter,
    pub min_filter: SamplerFilter,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn default() -> Arc<Self> {
        Self::with_color(
            Color::WHITE,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        )
    }

    pub fn new(
        name: &str,
        data: &[u8],
        extent: ImageExtent2D,
        ty: TextureType,
        format: ImageFormat,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            ty,
            name: name.to_string(),
            format,
            extent,
            mag_filter,
            min_filter,
            data: data.to_vec(),
        })
    }

    pub async fn with_file(
        file_name: &str,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Result<Arc<Self>> {
        let img = load::load_image(file_name, AssetType::IMAGE).await?;

        Ok(Self::new(
            file_name,
            &img.to_rgba8().into_raw(),
            ImageExtent2D {
                width: img.dimensions().0,
                height: img.dimensions().1,
            },
            ty,
            ty.into(),
            mag_filter,
            min_filter,
        ))
    }

    pub async fn pack(
        texture_files: &[&str],
        cols: u32,
        texture_type: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Result<Arc<Self>> {
        let n = texture_files.len();

        let (mut width, mut height) = (0, 0);

        let images = texture_files
            .iter()
            .map(|file| load::load_image(file, AssetType::IMAGE));

        let images = try_join_all(images).await?;

        for img in &images {
            if img.width() > width {
                width = img.width();
            }
            if img.height() > height {
                height = img.height();
            }
        }

        let mut target = ImageBuffer::new(cols * width, n as u32 / cols * height);

        for (i, img) in images.into_iter().enumerate() {
            let col = i as u32 % cols;
            let row = i as u32 / cols;

            target
                .copy_from(
                    &img.resize_to_fill(width, height, FilterType::Triangle)
                        .into_rgba8(),
                    col * width,
                    row * height,
                )
                .unwrap();
        }

        let img = &DynamicImage::ImageRgba8(target);

        Ok(Self::new(
            "texture_pack",
            &img.to_rgba8().into_raw(),
            ImageExtent2D::new(img.dimensions().0, img.dimensions().1),
            texture_type,
            texture_type.into(),
            mag_filter,
            min_filter,
        ))
    }

    pub fn with_colors(
        colors: &[Color],
        extent: ImageExtent2D,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Arc<Self> {
        Self::new(
            "framebuffer",
            &colors
                .iter()
                .flat_map(|c| Into::<[u8; 4]>::into(*c))
                .collect::<Vec<u8>>(),
            extent,
            ty,
            ty.into(),
            mag_filter,
            min_filter,
        )
    }

    pub fn with_color(
        color: Color,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Arc<Self> {
        let data: [u8; 4] = color.into();
        Self::new(
            "color texture",
            &data,
            ImageExtent2D::new(1, 1),
            ty,
            ty.into(),
            mag_filter,
            min_filter,
        )
    }

    const CHECKER_SIZE: usize = 8;
    pub fn with_checker(
        color1: Color,
        color2: Color,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Arc<Self> {
        let mut data: [u8; 4 * Self::CHECKER_SIZE * Self::CHECKER_SIZE] =
            [0; 4 * Self::CHECKER_SIZE * Self::CHECKER_SIZE];

        for y in 0..Self::CHECKER_SIZE {
            for x in 0..Self::CHECKER_SIZE {
                let color: [u8; 4] = if (x + y) % 2 == 0 {
                    color1.into()
                } else {
                    color2.into()
                };

                data[4 * y * Self::CHECKER_SIZE + 4 * x] = color[0];
                data[4 * y * Self::CHECKER_SIZE + 4 * x + 1] = color[1];
                data[4 * y * Self::CHECKER_SIZE + 4 * x + 2] = color[2];
                data[4 * y * Self::CHECKER_SIZE + 4 * x + 3] = color[3];
            }
        }

        Self::new(
            "checker",
            &data,
            ImageExtent2D::new(Self::CHECKER_SIZE as u32, Self::CHECKER_SIZE as u32),
            ty,
            ty.into(),
            mag_filter,
            min_filter,
        )
    }

    pub fn patch(
        &self,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Arc<Self> {
        let extent = self.extent;

        let mut new_data = self.data.clone();

        for x in 0..width {
            for y in 0..height {
                let local_idx = (x + y * width) as usize;
                let global_idx = ((start_x + x) + (start_y + y) * extent.width) as usize;

                new_data[4 * global_idx] = data[4 * local_idx];
                new_data[4 * global_idx + 1] = data[4 * local_idx + 1];
                new_data[4 * global_idx + 2] = data[4 * local_idx + 2];
                new_data[4 * global_idx + 3] = data[4 * local_idx + 3];
            }
        }

        Texture::new(
            &self.name,
            &new_data,
            extent,
            self.ty,
            self.format,
            self.mag_filter,
            self.min_filter,
        )
    }
}
