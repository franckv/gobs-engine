use futures::future::try_join_all;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, imageops::FilterType};
use pollster::FutureExt;

use gobs_core::{Color, ImageExtent2D, logger};
use gobs_render_graph::GfxContext;
use gobs_render_hal::{
    BufferType, CommandBuffer, CommandQueueType, ImageLayout, ImageUsage, RenderHAL,
};
use gobs_resource::{
    ResourceRegistry,
    load::{self, AssetType},
    {Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::resources::{BufferPool, Texture, TextureData, TextureFormat, texture::TexturePath};

pub struct TextureLoader {
    cmd: Box<dyn CommandBuffer>,
    buffer_pool: BufferPool,
}

impl TextureLoader {
    pub fn new(ctx: &mut GfxContext) -> Self {
        Self {
            cmd: ctx
                .hal_mut()
                .create_command_buffer("Mesh loader", CommandQueueType::Transfer),
            buffer_pool: BufferPool::new(),
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
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load<'a>(
        &mut self,
        hal: &mut (dyn RenderHAL + 'a),
        handle: &ResourceHandle<Texture>,
        registry: &mut ResourceRegistry,
    ) -> Result<TextureData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        tracing::debug!(target: logger::RESOURCES, "Load texture resource {}", properties.name());
        tracing::trace!(target: logger::RESOURCES, "Texture properties: {:?}", properties.format);

        let mut staging_data = vec![];

        Self::get_bytes(&properties.path, &mut properties.format, |data| {
            staging_data.extend_from_slice(data);
        });

        let staging = self.buffer_pool.allocate(
            hal,
            "image staging",
            staging_data.len(),
            BufferType::Staging,
        );
        hal.upload_buffer(staging.buffer, &staging_data, 0);

        let image_format = properties.format.ty.into();
        let image = hal.create_image(
            properties.name(),
            image_format,
            ImageUsage::Texture,
            properties.format.extent,
        );

        let sampler =
            hal.create_sampler(properties.format.mag_filter, properties.format.min_filter);

        self.cmd.run_immediate_mut("Texture upload", &mut |cmd| {
            cmd.transition_image_layout(hal, image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(hal, staging.buffer, image, 0);
            cmd.transition_image_layout(hal, image, ImageLayout::Shader);
        });

        Ok(TextureData {
            format: image_format,
            image,
            sampler,
        })
    }

    fn unload(&mut self, _resource: Resource<Texture>) {
        // drop resource
    }

    fn flush(&mut self) {
        self.buffer_pool.recycle_all();
    }
}
