use std::sync::Arc;

use ash::vk;

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
    pub fn new(device: Arc<Device>, descriptor_layout: Arc<DescriptorSetLayout>) -> Arc<Self> {
        let set_layout = [descriptor_layout.layout];

        let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layout);

        let layout = unsafe {
            PipelineLayout {
                device: device.clone(),
                descriptor_layout,
                layout: device
                    .raw()
                    .create_pipeline_layout(&layout_info, None)
                    .unwrap(),
            }
        };

        Arc::new(layout)
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
