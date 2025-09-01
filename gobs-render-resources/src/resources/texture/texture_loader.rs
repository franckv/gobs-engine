use std::sync::Arc;

use futures::future::try_join_all;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, imageops::FilterType};
use pollster::FutureExt;

use gobs_core::{Color, ImageExtent2D, logger, memory::allocator::Allocator};
use gobs_gfx::{
    Buffer, BufferUsage, Command, CommandQueueType, GfxBuffer, GfxCommand, GfxDevice, GfxImage,
    GfxSampler, Image, ImageLayout, ImageUsage, Sampler,
};
use gobs_resource::{
    load::{self, AssetType},
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::resources::{Texture, TextureData, TextureFormat, texture::TexturePath};

pub struct TextureLoader {
    device: Arc<GfxDevice>,
    pub buffer_pool: Allocator<GfxDevice, BufferUsage, GfxBuffer>,
    cmd: GfxCommand,
}

const STAGING_BUFFER_SIZE: usize = 1_048_576;

impl TextureLoader {
    pub fn new(device: Arc<GfxDevice>) -> Self {
        Self {
            device: device.clone(),
            buffer_pool: Allocator::new(),
            cmd: GfxCommand::new(&device, "Mesh loader", CommandQueueType::Transfer),
        }
    }

    fn load_file<F>(filename: &str, format: &mut TextureFormat, mut f: F)
    where
        F: FnMut(&[u8]),
    {
        tracing::debug!(target: logger::RESOURCES, "Load file: {:?}", &filename);

        let img = load::load_image(filename, AssetType::IMAGE);
        let img = img.block_on().unwrap();
        let data = &img.to_rgba8().into_raw();

        format.extent = ImageExtent2D::new(img.width(), img.height());

        f(data)
    }

    const CHECKER_SIZE: usize = 8;

    fn load_checker<F>(color1: Color, color2: Color, mut f: F)
    where
        F: FnMut(&[u8]),
    {
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

        f(&data)
    }

    fn load_color<F>(color: Color, mut f: F)
    where
        F: FnMut(&[u8]),
    {
        let data: [u8; 4] = color.into();

        f(&data)
    }

    fn load_colors<F>(colors: &[Color], mut f: F)
    where
        F: FnMut(&[u8]),
    {
        let data = &colors
            .iter()
            .flat_map(|c| Into::<[u8; 4]>::into(*c))
            .collect::<Vec<u8>>();

        // self.load_data("colors", data, format)
        f(data)
    }

    fn load_atlas<F>(texture_files: &[String], cols: u32, format: &mut TextureFormat, mut f: F)
    where
        F: FnMut(&[u8]),
    {
        let n = texture_files.len();

        let (mut width, mut height) = (0, 0);

        let images = texture_files
            .iter()
            .map(|file| load::load_image(file, AssetType::IMAGE));

        let images = try_join_all(images).block_on().unwrap();

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
        let data = &img.to_rgba8().into_raw();

        format.extent = ImageExtent2D::new(img.dimensions().0, img.dimensions().1);

        f(data);
    }

    fn load_default<F>(f: F)
    where
        F: FnMut(&[u8]),
    {
        Self::load_color(Color::WHITE, f);
    }

    pub fn get_bytes<F>(path: &TexturePath, format: &mut TextureFormat, mut f: F)
    where
        F: FnMut(&[u8]),
    {
        match path {
            TexturePath::Default => Self::load_default(f),
            TexturePath::File(filename) => Self::load_file(filename, format, f),
            TexturePath::Bytes(items) => f(items),
            TexturePath::Atlas(files, cols) => Self::load_atlas(files, *cols, format, f),
            TexturePath::Color(color) => Self::load_color(*color, f),
            TexturePath::Colors(colors) => Self::load_colors(colors, f),
            TexturePath::Checker(color1, color2) => Self::load_checker(*color1, *color2, f),
        }
    }
}

impl ResourceLoader<Texture> for TextureLoader {
    fn load(
        &mut self,
        handle: &ResourceHandle<Texture>,
        _: &(),
        registry: &mut ResourceRegistry,
    ) -> Result<TextureData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        tracing::debug!(target: logger::RESOURCES, "Load texture resource {}", properties.name());
        tracing::trace!(target: logger::RESOURCES, "Texture properties: {:?}", properties.format);

        let staging = self.buffer_pool.allocate(
            &self.device,
            "image staging",
            STAGING_BUFFER_SIZE,
            BufferUsage::Staging,
        )?;
        let staging_id = staging.id();

        Self::get_bytes(&properties.path, &mut properties.format, |data| {
            if data.len() > STAGING_BUFFER_SIZE {
                tracing::warn!("Resize staging buffer");
                staging.resize(data.len(), &self.device);
            }
            staging.copy(data, 0);
        });

        let image_format = properties.format.ty.into();
        let mut image = GfxImage::new(
            properties.name(),
            &self.device,
            image_format,
            ImageUsage::Texture,
            properties.format.extent,
        );
        let sampler = GfxSampler::new(
            &self.device,
            properties.format.mag_filter,
            properties.format.min_filter,
        );

        self.cmd.run_immediate_mut("Texture upload", |cmd| {
            let extent = image.extent();
            cmd.transition_image_layout(&mut image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(staging, &mut image, extent.width, extent.height);
            cmd.transition_image_layout(&mut image, ImageLayout::Shader);
        });

        self.buffer_pool.recycle(&staging_id);
        assert!(self.buffer_pool.is_empty());

        Ok(TextureData {
            format: image_format,
            image,
            sampler,
        })
    }

    fn unload(&mut self, _resource: Resource<Texture>) {
        // drop resource
    }
}
