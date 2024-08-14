use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use anyhow::Result;
use futures::future::try_join_all;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use uuid::Uuid;

use gobs_core::color::Color;
use gobs_gfx::{
    Buffer, BufferUsage, Command, Device, Image, ImageExtent2D, ImageFormat, ImageLayout,
    ImageUsage, Sampler, SamplerFilter,
};
use gobs_utils::load::{self, AssetType};

use crate::{context::Context, GfxBuffer, GfxImage, GfxSampler};

pub type TextureId = Uuid;

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Diffuse,
    Normal,
}

impl Into<ImageFormat> for TextureType {
    fn into(self) -> ImageFormat {
        match self {
            TextureType::Diffuse => ImageFormat::R8g8b8a8Srgb,
            TextureType::Normal => ImageFormat::R8g8b8a8Unorm,
        }
    }
}

#[derive(Clone)]
pub struct Texture(Arc<RwLock<TextureValue>>);

impl Texture {
    pub fn default(ctx: &Context) -> Self {
        Self::with_color(
            ctx,
            Color::WHITE,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        )
    }

    pub fn new(
        ctx: &Context,
        name: &str,
        data: &[u8],
        extent: ImageExtent2D,
        ty: TextureType,
        format: ImageFormat,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Self {
        let image = GfxImage::new(name, &ctx.device, format, ImageUsage::Texture, extent);

        let sampler = GfxSampler::new(&ctx.device, mag_filter, min_filter);

        let mut texture_value = TextureValue {
            id: Uuid::new_v4(),
            image,
            data: data.to_vec(),
            ty,
            sampler,
        };

        texture_value.upload_data(ctx);

        Self(Arc::new(RwLock::new(texture_value)))
    }

    pub async fn with_file(
        ctx: &Context,
        file_name: &str,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Result<Self> {
        let img = load::load_image(file_name, AssetType::IMAGE).await?;

        Ok(Self::new(
            ctx,
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
        ctx: &Context,
        texture_files: &[&str],
        cols: u32,
        texture_type: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Result<Self> {
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
            ctx,
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
        ctx: &Context,
        colors: &[Color],
        extent: ImageExtent2D,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Texture {
        Self::new(
            ctx,
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
        ctx: &Context,
        color: Color,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Texture {
        let data: [u8; 4] = color.into();
        Self::new(
            ctx,
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
        ctx: &Context,
        color1: Color,
        color2: Color,
        ty: TextureType,
        mag_filter: SamplerFilter,
        min_filter: SamplerFilter,
    ) -> Texture {
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
            ctx,
            "checker",
            &data,
            ImageExtent2D::new(Self::CHECKER_SIZE as u32, Self::CHECKER_SIZE as u32),
            ty,
            ty.into(),
            mag_filter,
            min_filter,
        )
    }

    pub fn read(&self) -> RwLockReadGuard<'_, TextureValue> {
        self.0.read().expect("Cannot read texture")
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, TextureValue> {
        self.0.write().expect("Cannot read texture")
    }

    pub fn patch(
        &self,
        ctx: &Context,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        let extent = self.0.read().unwrap().image.extent();

        {
            let new_data = &mut self.0.write().unwrap().data;

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
        }

        self.0.write().unwrap().upload_data(ctx);
    }
}

pub struct TextureValue {
    pub id: TextureId,
    pub image: GfxImage,
    pub data: Vec<u8>,
    pub ty: TextureType,
    pub sampler: GfxSampler,
}

impl TextureValue {
    fn upload_data(&mut self, ctx: &Context) {
        let mut staging = GfxBuffer::new(
            "image staging",
            self.data.len(),
            BufferUsage::Staging,
            &ctx.device,
        );

        staging.copy(&self.data, 0);

        ctx.device.run_immediate_mut(|cmd| {
            cmd.begin_label("Upload image");
            cmd.transition_image_layout(&mut self.image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(
                &staging,
                &self.image,
                self.image.extent().width,
                self.image.extent().height,
            );
            cmd.transition_image_layout(&mut self.image, ImageLayout::Shader);
            cmd.end_label();
        });
    }
}
