use anyhow::Result;
use futures::future::try_join_all;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use uuid::Uuid;

use gobs_core::color::Color;
use gobs_render::context::Context;
use gobs_utils::load::{self, AssetType};
use gobs_vulkan::buffer::{Buffer, BufferUsage};
use gobs_vulkan::image::{
    Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage, Sampler, SamplerFilter,
};

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

pub struct Texture {
    pub id: TextureId,
    pub image: Image,
    pub ty: TextureType,
    pub sampler: Sampler,
}

impl Texture {
    pub fn default(ctx: &Context) -> Self {
        Texture::with_color(
            ctx,
            Color::WHITE,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
        )
    }

    pub fn new(
        ctx: &Context,
        name: &str,
        data: &[u8],
        extent: ImageExtent2D,
        ty: TextureType,
        filter: SamplerFilter,
    ) -> Self {
        let mut image = Image::new(
            name,
            ctx.device.clone(),
            ty.into(),
            ImageUsage::Texture,
            extent,
            ctx.allocator.clone(),
        );

        let mut staging = Buffer::new(
            "image staging",
            data.len(),
            BufferUsage::Staging,
            ctx.device.clone(),
            ctx.allocator.clone(),
        );

        staging.copy(data, 0);

        ctx.immediate_cmd.immediate_mut(|cmd| {
            cmd.begin_label("Upload image");
            cmd.transition_image_layout(&mut image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(&staging, &image, extent.width, extent.height);
            cmd.transition_image_layout(&mut image, ImageLayout::Shader);
            cmd.end_label();
        });

        Self::with_image(ctx, image, ty, filter)
    }

    pub fn with_image(ctx: &Context, image: Image, ty: TextureType, filter: SamplerFilter) -> Self {
        let sampler = Sampler::new(ctx.device.clone(), filter);

        Texture {
            id: Uuid::new_v4(),
            image,
            ty,
            sampler,
        }
    }

    pub async fn with_file(
        ctx: &Context,
        file_name: &str,
        ty: TextureType,
        filter: SamplerFilter,
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
            filter,
        ))
    }

    pub async fn pack(
        ctx: &Context,
        texture_files: &[&str],
        cols: u32,
        texture_type: TextureType,
        filter: SamplerFilter,
    ) -> Result<Texture> {
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

        Ok(Texture::new(
            ctx,
            "texture_pack",
            &img.to_rgba8().into_raw(),
            ImageExtent2D::new(img.dimensions().0, img.dimensions().1),
            texture_type,
            filter,
        ))
    }

    pub fn with_colors(
        ctx: &Context,
        colors: Vec<Color>,
        extent: ImageExtent2D,
        ty: TextureType,
        filter: SamplerFilter,
    ) -> Self {
        Self::new(
            ctx,
            "framebuffer",
            &colors
                .iter()
                .flat_map(|c| Into::<[u8; 4]>::into(*c))
                .collect::<Vec<u8>>(),
            extent,
            ty,
            filter,
        )
    }

    pub fn with_color(ctx: &Context, color: Color, ty: TextureType, filter: SamplerFilter) -> Self {
        let data: [u8; 4] = color.into();
        Self::new(
            ctx,
            "color texture",
            &data,
            ImageExtent2D::new(1, 1),
            ty,
            filter,
        )
    }

    const CHECKER_SIZE: usize = 8;
    pub fn with_checker(
        ctx: &Context,
        color1: Color,
        color2: Color,
        ty: TextureType,
        filter: SamplerFilter,
    ) -> Self {
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
            filter,
        )
    }
}
