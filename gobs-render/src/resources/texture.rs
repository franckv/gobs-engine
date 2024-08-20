use std::sync::Arc;

use gobs_gfx::{Buffer, BufferUsage, Command, Device, Image, ImageLayout, ImageUsage, Sampler};
use gobs_resource::material::Texture;

use crate::context::Context;
use crate::{GfxBuffer, GfxImage, GfxSampler};

pub struct GpuTexture {
    pub texture: Arc<Texture>,
    pub image: GfxImage,
    pub sampler: GfxSampler,
}

impl GpuTexture {
    pub fn new(ctx: &Context, texture: Arc<Texture>) -> Self {
        let mut image = GfxImage::new(
            &texture.name,
            &ctx.device,
            texture.format,
            ImageUsage::Texture,
            texture.extent,
        );

        let sampler = GfxSampler::new(&ctx.device, texture.mag_filter, texture.min_filter);

        Self::upload_data(ctx, &texture.data, &mut image);

        Self {
            texture,
            image,
            sampler,
        }
    }

    pub fn image(&self) -> &GfxImage {
        &self.image
    }

    pub fn sampler(&self) -> &GfxSampler {
        &self.sampler
    }

    fn upload_data(ctx: &Context, data: &[u8], image: &mut GfxImage) {
        let mut staging = GfxBuffer::new(
            "image staging",
            data.len(),
            BufferUsage::Staging,
            &ctx.device,
        );

        staging.copy(data, 0);

        ctx.device.run_immediate_mut(|cmd| {
            cmd.begin_label("Upload image");
            cmd.transition_image_layout(image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(&staging, image, image.extent().width, image.extent().height);
            cmd.transition_image_layout(image, ImageLayout::Shader);
            cmd.end_label();
        });
    }
}
