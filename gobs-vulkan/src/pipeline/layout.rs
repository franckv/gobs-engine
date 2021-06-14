use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use log::trace;

use crate::descriptor::DescriptorSetLayout;
use crate::device::Device;
use crate::Wrap;

pub struct PipelineLayout {
    device: Arc<Device>,
    descriptor_layout: Arc<DescriptorSetLayout>,
    pub(crate) layout: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(device: Arc<Device>, descriptor_layout: Arc<DescriptorSetLayout>) -> Self {
        let set_layout = [descriptor_layout.layout];

        let layout_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            set_layout_count: 1,
            p_set_layouts: set_layout.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        unsafe {
            PipelineLayout {
                device: device.clone(),
                descriptor_layout,
                layout: device.raw().create_pipeline_layout(&layout_info, None).unwrap()
            }
        }
    }
}

impl Wrap<vk::PipelineLayout> for PipelineLayout {
    fn raw(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        trace!("Drop pipeline layout");
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.layout, None);
        }
    }
}
