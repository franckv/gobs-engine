use gobs_core::{Color, ImageExtent2D, ImageFormat, SamplerFilter};
use gobs_gfx::{
    Buffer, BufferUsage, Command, Device, GfxBuffer, GfxDevice, GfxImage, GfxSampler, Image,
    ImageLayout, ImageUsage, Sampler,
};
use gobs_resource::resource::{Resource, ResourceHandle, ResourceType};

use super::TextureLoader;

pub struct Texture;

impl ResourceType for Texture {
    type ResourceData = TextureData;
    type ResourceProperties = TextureProperties;
    type ResourceLoader = TextureLoader;
}

#[derive(Clone, Debug)]
pub enum TexturePath {
    Default,
    File(String),
    Atlas(Vec<String>, u32),
    Bytes(Vec<u8>),
    Color(Color),
    Colors(Vec<Color>),
    Checker(Color, Color),
}

#[derive(Clone, Debug)]
pub struct TextureFormat {
    pub ty: TextureType,
    pub extent: ImageExtent2D,
    pub mag_filter: SamplerFilter,
    pub min_filter: SamplerFilter,
}

#[derive(Clone, Debug)]
pub struct TextureProperties {
    pub name: String,
    pub path: TexturePath,
    pub format: TextureFormat,
}

impl TextureProperties {
    pub fn with_data(name: &str, data: Vec<u8>, extent: ImageExtent2D) -> Self {
        Self {
            name: name.to_string(),
            path: TexturePath::Bytes(data),
            format: TextureFormat {
                ty: TextureType::Diffuse,
                extent,
                mag_filter: SamplerFilter::FilterLinear,
                min_filter: SamplerFilter::FilterLinear,
            },
        }
    }

    pub fn with_file(name: &str, filename: &str) -> Self {
        Self {
            name: name.to_string(),
            path: TexturePath::File(filename.to_string()),
            format: TextureFormat {
                ty: TextureType::Diffuse,
                extent: ImageExtent2D::new(0, 0),
                mag_filter: SamplerFilter::FilterLinear,
                min_filter: SamplerFilter::FilterLinear,
            },
        }
    }

    pub fn with_atlas(name: &str, filenames: &[&str], cols: u32) -> Self {
        Self {
            name: name.to_string(),
            path: TexturePath::Atlas(filenames.iter().map(|&f| f.to_string()).collect(), cols),
            format: TextureFormat {
                ty: TextureType::Diffuse,
                extent: ImageExtent2D::new(1, 1),
                mag_filter: SamplerFilter::FilterLinear,
                min_filter: SamplerFilter::FilterLinear,
            },
        }
    }

    pub fn with_color(name: &str, color: Color) -> Self {
        Self {
            name: name.to_string(),
            path: TexturePath::Color(color),
            format: TextureFormat {
                ty: TextureType::Diffuse,
                extent: ImageExtent2D::new(1, 1),
                mag_filter: SamplerFilter::FilterLinear,
                min_filter: SamplerFilter::FilterLinear,
            },
        }
    }

    pub fn with_colors(name: &str, colors: Vec<Color>, extent: ImageExtent2D) -> Self {
        Self {
            name: name.to_string(),
            path: TexturePath::Colors(colors),
            format: TextureFormat {
                ty: TextureType::Diffuse,
                extent,
                mag_filter: SamplerFilter::FilterLinear,
                min_filter: SamplerFilter::FilterLinear,
            },
        }
    }
}

impl Default for TextureProperties {
    fn default() -> Self {
        Self {
            name: "Default texture".to_string(),
            path: TexturePath::Default,
            format: TextureFormat {
                ty: TextureType::Diffuse,
                extent: ImageExtent2D::new(0, 0),
                mag_filter: SamplerFilter::FilterLinear,
                min_filter: SamplerFilter::FilterLinear,
            },
        }
    }
}

pub trait TextureUpdate {
    fn patch(
        &mut self,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> ResourceHandle;
}

impl TextureUpdate for Resource<Texture> {
    fn patch(
        &mut self,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> ResourceHandle {
        if let TexturePath::Bytes(old_data) = &self.properties.path {
            let extent = self.properties.format.extent;

            // let mut new_data = vec![];
            // new_data.extend_from_slice(&old_data);
            let mut new_data = old_data.clone();

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

            self.properties.path = TexturePath::Bytes(new_data);
        } else {
            tracing::error!("Cannot patch resource: self.source");
        }

        self.id
    }
}

pub struct TextureData {
    pub format: ImageFormat,
    pub image: GfxImage,
    pub sampler: GfxSampler,
}

impl TextureData {
    pub(crate) fn new(device: &GfxDevice, name: &str, format: &TextureFormat, data: &[u8]) -> Self {
        tracing::debug!("Texture properties: {:?}", format);

        let image_format = format.ty.into();
        let mut image = GfxImage::new(
            name,
            device,
            image_format,
            ImageUsage::Texture,
            format.extent,
        );
        let sampler = GfxSampler::new(device, format.mag_filter, format.min_filter);

        Self::upload_data(device, data, &mut image);

        Self {
            format: image_format,
            image,
            sampler,
        }
    }

    fn upload_data(device: &GfxDevice, data: &[u8], image: &mut GfxImage) {
        let mut staging = GfxBuffer::new("image staging", data.len(), BufferUsage::Staging, device);

        staging.copy(data, 0);

        device.run_transfer_mut(|cmd| {
            cmd.begin_label("Upload image");
            cmd.transition_image_layout(image, ImageLayout::TransferDst);
            cmd.copy_buffer_to_image(&staging, image, image.extent().width, image.extent().height);
            cmd.transition_image_layout(image, ImageLayout::Shader);
            cmd.end_label();
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Diffuse,
    Normal,
}

impl From<TextureType> for ImageFormat {
    fn from(val: TextureType) -> Self {
        match val {
            TextureType::Diffuse => ImageFormat::R8g8b8a8Srgb,
            TextureType::Normal => ImageFormat::R8g8b8a8Unorm,
        }
    }
}
