use gobs_core::ImageExtent2D;
use gobs_gfx::{Image, ImageFormat, ImageUsage, Sampler, SamplerFilter};
use gobs_vulkan as vk;

use crate::VkDevice;

pub struct VkImage {
    pub(crate) image: vk::image::Image,
}

impl Image for VkImage {
    type GfxDevice = VkDevice;

    fn new(
        name: &str,
        device: &VkDevice,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self {
        Self {
            image: vk::image::Image::new(
                name,
                device.device.clone(),
                format,
                usage,
                extent,
                device.allocator.clone(),
            ),
        }
    }

    fn invalidate(&mut self) {
        self.image.invalidate();
    }

    fn extent(&self) -> ImageExtent2D {
        self.image.extent
    }

    fn name(&self) -> &str {
        &self.image.label
    }

    fn format(&self) -> ImageFormat {
        self.image.format
    }
}

impl VkImage {
    pub(crate) fn from_raw(image: vk::image::Image) -> Self {
        Self { image }
    }
}

pub struct VkSampler {
    pub(crate) sampler: vk::image::Sampler,
    mag_filter: SamplerFilter,
    min_filter: SamplerFilter,
}

impl Sampler for VkSampler {
    type GfxDevice = VkDevice;

    fn new(device: &VkDevice, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self {
        Self {
            sampler: vk::image::Sampler::new(device.device.clone(), mag_filter, min_filter),
            mag_filter,
            min_filter,
        }
    }

    fn mag_filter(&self) -> SamplerFilter {
        self.mag_filter
    }

    fn min_filter(&self) -> SamplerFilter {
        self.min_filter
    }
}
