use std::sync::Arc;

use ash::vk;

use crate::descriptor::DescriptorSetLayout;
use crate::device::Device;
use crate::Wrap;

#[allow(unused)]
pub struct PipelineLayout {
    device: Arc<Device>,
    descriptor_layout: Option<Arc<DescriptorSetLayout>>,
    pub(crate) layout: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(
        device: Arc<Device>,
        descriptor_layout: Option<Arc<DescriptorSetLayout>>,
    ) -> Arc<Self> {
        let mut layout_info = vk::PipelineLayoutCreateInfo::builder();

        let mut set_layout = vec![];

        if let Some(descriptor_layout) = &descriptor_layout {
            set_layout.push(descriptor_layout.layout);
            layout_info = layout_info.set_layouts(&set_layout);
        }

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

    pub fn with_constants(
        device: Arc<Device>,
        descriptor_layout: Option<Arc<DescriptorSetLayout>>,
        size: usize,
    ) -> Arc<Self> {
        let push_constant_range = vec![vk::PushConstantRange::builder()
            .offset(0)
            .size(size as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];

        let mut layout_info =
            vk::PipelineLayoutCreateInfo::builder().push_constant_ranges(&push_constant_range);

        let mut set_layout = vec![];

        if let Some(descriptor_layout) = &descriptor_layout {
            set_layout.push(descriptor_layout.layout);
            layout_info = layout_info.set_layouts(&set_layout);
        }

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
        log::debug!("Drop pipeline layout");
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.layout, None);
        }
    }
}
