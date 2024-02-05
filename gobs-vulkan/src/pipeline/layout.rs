use std::sync::Arc;

use ash::vk;

use crate::descriptor::DescriptorSetLayout;
use crate::device::Device;
use crate::Wrap;

#[allow(unused)]
pub struct PipelineLayout {
    device: Arc<Device>,
    descriptor_layouts: Vec<Arc<DescriptorSetLayout>>,
    pub(crate) layout: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(
        device: Arc<Device>,
        descriptor_layouts: &[Arc<DescriptorSetLayout>],
        push_constant_size: usize,
    ) -> Arc<Self> {
        let mut layout_info = vk::PipelineLayoutCreateInfo::builder();

        let mut set_layout = vec![];
        for descriptor_layout in descriptor_layouts {
            set_layout.push(descriptor_layout.layout);
        }
        if !set_layout.is_empty() {
            layout_info = layout_info.set_layouts(&set_layout);
        }

        let mut push_constant_range = vec![];
        if push_constant_size > 0 {
            debug_assert!(push_constant_size <= 128);
            push_constant_range.push(
                vk::PushConstantRange::builder()
                    .offset(0)
                    .size(push_constant_size as u32)
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .build(),
            );
            layout_info = layout_info.push_constant_ranges(&push_constant_range);
        }

        let layout = unsafe {
            PipelineLayout {
                device: device.clone(),
                descriptor_layouts: descriptor_layouts.to_vec(),
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
        log::debug!("Drop pipeline layout");
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.layout, None);
        }
    }
}
