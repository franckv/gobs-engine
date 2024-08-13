use gobs_vulkan as vk;

use crate::backend::vulkan::VkDevice;
use crate::{Image, ImageExtent2D, ImageFormat, ImageUsage, Sampler, SamplerFilter};

pub struct VkImage {
    pub(crate) image: vk::image::Image,
}

impl Image for VkImage {
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
}

impl VkImage {
    pub(crate) fn from_raw(image: vk::image::Image) -> Self {
        Self { image }
    }
}

pub struct VkSampler {
    pub(crate) sampler: vk::image::Sampler,
}

impl Sampler for VkSampler {
    fn new(device: &VkDevice, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self {
        Self {
            sampler: vk::image::Sampler::new(device.device.clone(), mag_filter, min_filter),
        }
    }
}
