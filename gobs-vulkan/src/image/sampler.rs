use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::device::Device;
use crate::Wrap;

pub struct Sampler {
    device: Arc<Device>,
    sampler: vk::Sampler
}

impl Sampler {
    pub fn new(device: Arc<Device>) -> Self {
        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: 0,
            max_anisotropy: 1.,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: 0,
            compare_enable: 0,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.,
            min_lod: 0.,
            max_lod: 0.
        };

        let sampler = unsafe {
            device.raw().create_sampler(&sampler_info,
                                        None).unwrap()
        };

        Sampler {
            device,
            sampler
        }
    }
}

impl Wrap<vk::Sampler> for Sampler {
    fn raw(&self) -> vk::Sampler {
        self.sampler
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        trace!("Drop sampler");
        unsafe {
            self.device.raw().destroy_sampler(self.sampler, None);
        }
    }
}
