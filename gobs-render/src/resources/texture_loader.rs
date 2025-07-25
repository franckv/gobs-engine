use std::sync::Arc;

use futures::future::try_join_all;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, imageops::FilterType};
use pollster::FutureExt;

use gobs_core::{Color, ImageExtent2D, logger};
use gobs_gfx::{
    Buffer, BufferUsage, Command, CommandQueueType, GfxBuffer, GfxCommand, GfxDevice, GfxImage,
    GfxSampler, Image, ImageLayout, ImageUsage, Sampler,
};
use gobs_resource::{
    load::{self, AssetType},
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader},
};

use crate::{
    Texture,
    resources::{
        TextureData, TextureProperties,
        texture::{TextureFormat, TexturePath},
    },
};

pub struct TextureLoader {
    device: Arc<GfxDevice>,
    cmd: GfxCommand,
}

impl TextureLoader {
    pub fn new(device: Arc<GfxDevice>) -> Self {
        Self {
            device: device.clone(),
            cmd: GfxCommand::new(&device, "Mesh loader", CommandQueueType::Transfer),
        }
    }

    fn load_file(&self, filename: &str, format: &mut TextureFormat) -> TextureData {
        tracing::debug!(target: logger::RESOURCES, "Load file: {:?}", &filename);

        let img = load::load_image(filename, AssetType::IMAGE);
        let img = img.block_on().unwrap();
        let data = &img.to_rgba8().into_raw();

        format.extent = ImageExtent2D::new(img.width(), img.height());

        self.load_data(filename, data, format)
    }

    fn load_data(&self, name: &str, data: &[u8], format: &TextureFormat) -> TextureData {
        tracing::debug!(target: logger::RESOURCES, "Load texture data: {:?}", name);

        tracing::trace!(target: logger::RESOURCES, "Texture properties: {:?}", format);

        let image_format = format.ty.into();
        let mut image = GfxImage::new(
            name,
            &self.device,
            image_format,
            ImageUsage::Texture,
            format.extent,
        );
        let sampler = GfxSampler::new(&self.device, format.mag_filter, format.min_filter);

        self.upload_data(&self.device, data, &mut image);

        TextureData {
            format: image_format,
            image,
            sampler,
        }
    }

    const CHECKER_SIZE: usize = 8;
    fn load_checker(&self, color1: Color, color2: Color, format: &TextureFormat) -> TextureData {
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

        self.load_data("checker", &data, format)
    }

    fn load_color(&self, color: Color, format: &TextureFormat) -> TextureData {
        let data: [u8; 4] = color.into();

        self.load_data("color", &data, format)
    }

    fn load_colors(&self, colors: &[Color], format: &TextureFormat) -> TextureData {
        let data = &colors
            .iter()
            .flat_map(|c| Into::<[u8; 4]>::into(*c))
            .collect::<Vec<u8>>();

        self.load_data("colors", data, format)
    }

    fn load_atlas(
        &self,
        texture_files: &[String],
        cols: u32,
        format: &mut TextureFormat,
    ) -> TextureData {
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

        self.load_data("pack", data, format)
    }

    fn load_default(&self) -> TextureData {
        let properties = TextureProperties::default();

        self.load_color(Color::WHITE, &properties.format)
    }

    fn upload_data(&self, device: &GfxDevice, data: &[u8], image: &mut GfxImage) {
        let mut staging = GfxBuffer::new("image staging", data.len(), BufferUsage::Staging, device);

        staging.copy(data, 0);

        self.cmd.run_immediate_mut("Texture upload", |cmd| {
            cmd.transition_image_layout(image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(&staging, image, image.extent().width, image.extent().height);
            cmd.transition_image_layout(image, ImageLayout::Shader);
        });
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

        let data = match &properties.path {
            TexturePath::Default => self.load_default(),
            TexturePath::File(filename) => self.load_file(filename, &mut properties.format),
            TexturePath::Bytes(data) => self.load_data(&properties.name, data, &properties.format),
            TexturePath::Atlas(files, cols) => {
                self.load_atlas(files, *cols, &mut properties.format)
            }
            TexturePath::Color(color) => self.load_color(*color, &properties.format),
            TexturePath::Colors(colors) => self.load_colors(colors, &properties.format),
            TexturePath::Checker(color1, color2) => {
                self.load_checker(*color1, *color2, &properties.format)
            }
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<Texture>) {
        // drop resource
    }
}
