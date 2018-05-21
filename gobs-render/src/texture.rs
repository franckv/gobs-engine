use std::sync::Arc;
use std::vec::Vec;

use image;
use image::{ImageBuffer, GenericImage, Pixel};

use vulkano::device::{Device, Queue};
use vulkano::format::R8G8B8A8Srgb;
use vulkano::image::Dimensions::Dim2d;
use vulkano::image::immutable::ImmutableImage;
use vulkano::sampler;
use vulkano::sampler::Sampler;
use vulkano::sync::GpuFuture;

use color::Color;
use render::Renderer;

pub struct TextureLoader {
    queue: Arc<Queue>
}

impl TextureLoader {
    pub fn new(renderer: &Renderer) -> TextureLoader {
        TextureLoader {
            queue: renderer.queue()
        }
    }

    pub fn load_texture(&self, path: &str) -> Texture {
        Texture::new(path, self.queue.clone())
    }

    pub fn load_texture_raw(&self, raw: &Vec<u8>, width: usize, height: usize) -> Texture {
        Texture::from_raw(raw, width, height, self.queue.clone())
    }

    pub fn load_color(&self, color: Color) -> Texture {
        Texture::create_color(color, self.queue.clone())
    }
}

pub struct Texture {
    image: Arc<ImmutableImage<R8G8B8A8Srgb>>,
    sampler: Arc<Sampler>,
    size: [usize; 2]
}

impl Texture {
    pub fn new(path: &str, queue: Arc<Queue>) -> Texture {
        let img = image::open(path).expect("Texture file not found");

        let (width, height) = (img.width() as usize, img.height() as usize);

        let img_data = img.to_rgba().into_raw();

        Self::from_raw(&img_data, width, height, queue)
    }

    pub fn create_color(color: Color, queue: Arc<Queue>) -> Texture {
        let c: [u8; 4] = color.into();
        let pixel = image::Rgba::from_channels(c[0], c[1], c[2], c[3]);

        let img = ImageBuffer::from_pixel(1, 1, pixel);

        let img_data = img.into_raw();

        Self::from_raw(&img_data, 1, 1, queue)
    }

    fn from_raw(img_data: &Vec<u8>, width: usize, height: usize, queue: Arc<Queue>)
    -> Texture {
        let (image, future) = ImmutableImage::from_iter(img_data.iter().cloned(),
        Dim2d { width: width as u32, height: height as u32},
        R8G8B8A8Srgb,
        queue.clone()).expect("Failed to load texture");

        future.flush().unwrap();

        let dim = image.dimensions().width_height();

        Texture {
            image: image,
            sampler: Self::create_sampler(queue.device().clone()),
            size: [dim[0] as usize, dim[1] as usize]
        }
    }

    fn create_sampler(device: Arc<Device>) -> Arc<Sampler> {
        sampler::Sampler::new(device.clone(),
            sampler::Filter::Nearest,
            sampler::Filter::Nearest,
            sampler::MipmapMode::Linear,
            sampler::SamplerAddressMode::ClampToEdge,
            sampler::SamplerAddressMode::ClampToEdge,
            sampler::SamplerAddressMode::ClampToEdge,
            0.0, 1.0, 0.0, 1.0).unwrap()
    }

    pub fn image(&self) -> Arc<ImmutableImage<R8G8B8A8Srgb>> {
        self.image.clone()
    }

    pub fn sampler(&self) -> Arc<Sampler> {
        self.sampler.clone()
    }

    pub fn size(&self) -> &[usize; 2] {
        &self.size
    }
}
