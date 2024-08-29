use std::sync::Arc;

use gobs_gfx::{
    Buffer, BufferUsage, Command, Device, Image, ImageLayout, ImageUsage, Renderer, Sampler,
};
use gobs_resource::material::Texture;

use crate::context::Context;

pub struct GpuTexture<R: Renderer> {
    pub _texture: Arc<Texture>,
    pub image: R::Image,
    pub sampler: R::Sampler,
}

impl<R: Renderer> GpuTexture<R> {
    pub fn new(ctx: &Context<R>, texture: Arc<Texture>) -> Self {
        let mut image = R::Image::new(
            &texture.name,
            &ctx.device,
            texture.format,
            ImageUsage::Texture,
            texture.extent,
        );

        let sampler = R::Sampler::new(&ctx.device, texture.mag_filter, texture.min_filter);

        Self::upload_data(ctx, &texture.data, &mut image);

        Self {
            _texture: texture,
            image,
            sampler,
        }
    }

    pub fn image(&self) -> &R::Image {
        &self.image
    }

    pub fn sampler(&self) -> &R::Sampler {
        &self.sampler
    }

    fn upload_data(ctx: &Context<R>, data: &[u8], image: &mut R::Image) {
        let mut staging = R::Buffer::new(
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
