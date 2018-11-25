use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use backend::device::Device;
use backend::Wrap;

pub struct Sampler {
    device: Arc<Device>,
    sampler: vk::Sampler
}

impl Sampler {
    pub fn new(device: Arc<Device>) -> Self {
        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SamplerCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            mag_filter: vk::Filter::Linear,
            min_filter: vk::Filter::Linear,
            address_mode_u: vk::SamplerAddressMode::Repeat,
            address_mode_v: vk::SamplerAddressMode::Repeat,
            address_mode_w: vk::SamplerAddressMode::Repeat,
            anisotropy_enable: 0,
            max_anisotropy: 1.,
            border_color: vk::BorderColor::IntOpaqueBlack,
            unnormalized_coordinates: 0,
            compare_enable: 0,
            compare_op: vk::CompareOp::Always,
            mipmap_mode: vk::SamplerMipmapMode::Linear,
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