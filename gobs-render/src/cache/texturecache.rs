use std::sync::Arc;
use std::collections::HashMap;

use vulkano::format::R8G8B8A8Srgb;
use vulkano::image::Dimensions::Dim2d;
use vulkano::image::immutable::ImmutableImage;
use vulkano::sampler;
use vulkano::sampler::Sampler;
use vulkano::sync::GpuFuture;

use uuid::Uuid;

use context::Context;
use scene::model::Texture;

pub struct TextureCache {
    context: Arc<Context>,
    cache: HashMap<Uuid, TextureCacheEntry>
}

impl TextureCache {
    pub fn new(context: Arc<Context>) -> Self {
        TextureCache {
            context: context,
            cache: HashMap::new()
        }
    }

    pub fn get(&mut self, texture: Arc<Texture>) -> &TextureCacheEntry {
        let id = texture.id();

        if !self.cache.contains_key(&id) {
            let entry = TextureCacheEntry::new(texture, self.context.clone());
            self.cache.insert(id, entry);
        }

        self.cache.get(&id).unwrap()
    }
}

pub struct TextureCacheEntry {
    image: Arc<ImmutableImage<R8G8B8A8Srgb>>,
    sampler: Arc<Sampler>
}

impl TextureCacheEntry {
    pub fn new(texture: Arc<Texture>, context: Arc<Context>) -> Self {
        let [width, height] = texture.size();

        let (image, future) = ImmutableImage::from_iter(texture.data().iter().cloned(),
        Dim2d { width: width as u32, height: height as u32},
        R8G8B8A8Srgb,
        context.queue()).expect("Failed to load texture");

        println!("Loading");

        future.flush().unwrap();

        TextureCacheEntry {
            image: image,
            sampler: Self::create_sampler(context),
        }
    }

    fn create_sampler(context: Arc<Context>) -> Arc<Sampler> {
        sampler::Sampler::new(context.device(),
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
}

impl Drop for TextureCacheEntry {
    fn drop(&mut self) {
        println!("Dropping");
    }
}
