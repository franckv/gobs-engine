use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::backend::device::Device;
use crate::backend::pipeline::PipelineLayout;

pub struct DescriptorSetLayout {
    device: Arc<Device>,
    pub(crate) layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(device: Arc<Device>, pipeline_layout: &PipelineLayout) -> Arc<Self> {
        let bindings: Vec<vk::DescriptorSetLayoutBinding> =
            pipeline_layout.bindings.iter().enumerate().
                map(|(idx, binding)| {
                    vk::DescriptorSetLayoutBinding {
                        binding: idx as u32,
                        descriptor_type: binding.ty.into(),
                        descriptor_count: 1,
                        p_immutable_samplers: ptr::null(),
                        stage_flags: binding.stage.into()
                    }
                }).collect();

        let descriptor_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
        };

        let layout = unsafe {
            device.raw().create_descriptor_set_layout(&descriptor_info,
                                                      None).unwrap()
        };

        Arc::new(DescriptorSetLayout {
            device,
            layout,
        })
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_descriptor_set_layout(self.layout,
                                                            None);
        }
    }
}
