use gobs_core::{ImageExtent2D, ImageFormat, SamplerFilter};
use gobs_vulkan as vk;

use crate::backend::vulkan::{device::VkDevice, renderer::VkRenderer};
use crate::{Image, ImageUsage, Sampler};

pub struct VkImage {
    pub(crate) image: vk::images::Image,
}

impl Image<VkRenderer> for VkImage {
    fn new(
        name: &str,
        device: &VkDevice,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self {
        Self {
            image: vk::images::Image::new(
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

    fn usage(&self) -> ImageUsage {
        self.image.usage
    }

    fn size(&self) -> usize {
        let extent = self.extent();

        let pixel_size = self.image.format.pixel_size();

        (extent.width * extent.height * pixel_size) as usize
    }
}

impl VkImage {
    pub(crate) fn from_raw(image: vk::images::Image) -> Self {
        Self { image }
    }
}

pub struct VkSampler {
    pub(crate) sampler: vk::images::Sampler,
    mag_filter: SamplerFilter,
    min_filter: SamplerFilter,
}

impl Sampler<VkRenderer> for VkSampler {
    fn new(device: &VkDevice, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self {
        Self {
            sampler: vk::images::Sampler::new(device.device.clone(), mag_filter, min_filter),
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
