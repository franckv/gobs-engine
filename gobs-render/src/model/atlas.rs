use anyhow::Result;
use image::{imageops::FilterType, DynamicImage, GenericImage, GenericImageView, ImageBuffer};

use gobs_core as core;
use gobs_utils as utils;

use core::material::texture::{Texture, TextureType};
use utils::load::{self, AssetType};

pub async fn load_atlas(files: &[&str], cols: u32, is_normal_map: bool) -> Result<Texture> {
    let n = files.len();

    let mut imgs = Vec::new();
    let (mut width, mut height) = (0, 0);

    for file in files.iter() {
        let img: DynamicImage = load::load_image(file, AssetType::IMAGE).await?;

        if img.width() > width {
            width = img.width();
        }
        if img.height() > height {
            height = img.height();
        }

        imgs.push(img);
    }

    let mut target = ImageBuffer::new(cols * width, n as u32 / cols * height);

    for (i, img) in imgs.into_iter().enumerate() {
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

    let ty = match is_normal_map {
        true => TextureType::NORMAL,
        false => TextureType::IMAGE,
    };

    let img = &DynamicImage::ImageRgba8(target);

    Ok(Texture::new(
        "atlas",
        ty,
        img.to_rgba8().into_raw(),
        img.dimensions().0,
        img.dimensions().1,
    ))
}
