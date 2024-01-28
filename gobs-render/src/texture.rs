use uuid::Uuid;

use gobs_core::color::Color;
use gobs_vulkan::buffer::{Buffer, BufferUsage};
use gobs_vulkan::image::{Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage};

use crate::context::Context;

pub type TextureId = Uuid;

pub struct Texture {
    pub id: TextureId,
    pub image: Image,
}

impl Texture {
    pub fn new(ctx: &Context, name: &str, data: &[u8], extent: ImageExtent2D) -> Self {
        let mut image = Image::new(
            name,
            ctx.device.clone(),
            ImageFormat::R8g8b8a8Unorm,
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

        Self::with_image(image)
    }

    pub fn with_image(image: Image) -> Self {
        Texture {
            id: Uuid::new_v4(),
            image,
        }
    }

    pub fn with_color(ctx: &Context, color: Color) -> Self {
        let data: [u8; 4] = color.into();
        Self::new(ctx, "color", &data, ImageExtent2D::new(1, 1))
    }

    const CHECKER_SIZE: usize = 16;
    pub fn with_checker(ctx: &Context, color1: Color, color2: Color) -> Self {
        let mut data: [u8; 4 * Self::CHECKER_SIZE * Self::CHECKER_SIZE] =
            [0; 4 * Self::CHECKER_SIZE * Self::CHECKER_SIZE];

        for y in 0..Self::CHECKER_SIZE {
            for x in 0..Self::CHECKER_SIZE {
                let color: [u8; 4] = if (x + y) % 2 == 0 {
                    color1.into()
                } else {
                    color2.into()
                };

                data[y * Self::CHECKER_SIZE + 4 * x] = color[0];
                data[y * Self::CHECKER_SIZE + 4 * x + 1] = color[1];
                data[y * Self::CHECKER_SIZE + 4 * x + 2] = color[2];
                data[y * Self::CHECKER_SIZE + 4 * x + 3] = color[3];
            }
        }

        Self::new(ctx, "checker", &data, ImageExtent2D::new(16, 16))
    }
}
