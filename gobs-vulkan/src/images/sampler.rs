use std::sync::Arc;

use ash::vk;

use gobs_core::{SamplerFilter, logger};

use crate::Wrap;
use crate::device::Device;

pub struct VkFilter(vk::Filter);

impl From<SamplerFilter> for VkFilter {
    fn from(value: SamplerFilter) -> VkFilter {
        match value {
            SamplerFilter::FilterNearest => VkFilter(vk::Filter::NEAREST),
            SamplerFilter::FilterLinear => VkFilter(vk::Filter::LINEAR),
        }
    }
}

impl From<VkFilter> for vk::Filter {
    fn from(value: VkFilter) -> Self {
        value.0
    }
}

pub struct Sampler {
    device: Arc<Device>,
    pub sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: Arc<Device>, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(VkFilter::from(mag_filter).into())
            .min_filter(VkFilter::from(min_filter).into());

        let sampler = unsafe { device.raw().create_sampler(&sampler_info, None).unwrap() };

        Sampler { device, sampler }
    }
}

impl Wrap<vk::Sampler> for Sampler {
    fn raw(&self) -> vk::Sampler {
        self.sampler
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop sampler");
        unsafe {
            self.device.raw().destroy_sampler(self.sampler, None);
        }
    }
}
