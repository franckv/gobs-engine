use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::Wrap;

#[derive(Clone, Copy, Debug)]
pub enum SamplerFilter {
    FilterNearest,
    FilterLinear,
}

impl Into<vk::Filter> for SamplerFilter {
    fn into(self) -> vk::Filter {
        match self {
            SamplerFilter::FilterNearest => vk::Filter::NEAREST,
            SamplerFilter::FilterLinear => vk::Filter::LINEAR,
        }
    }
}

pub struct Sampler {
    device: Arc<Device>,
    sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: Arc<Device>, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(mag_filter.into())
            .min_filter(min_filter.into());

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
        log::debug!("Drop sampler");
        unsafe {
            self.device.raw().destroy_sampler(self.sampler, None);
        }
    }
}
