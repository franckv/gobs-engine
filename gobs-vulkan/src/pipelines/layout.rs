use std::sync::Arc;

use ash::vk;

use gobs_core::logger;

use crate::Wrap;
use crate::descriptor::DescriptorSetLayout;
use crate::device::Device;

#[derive(Debug)]
pub struct PipelineLayout {
    device: Arc<Device>,
    _descriptor_layouts: Vec<Arc<DescriptorSetLayout>>,
    pub(crate) layout: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(
        device: Arc<Device>,
        mut descriptor_layouts: Vec<Arc<DescriptorSetLayout>>,
        push_constant_size: usize,
    ) -> Arc<Self> {
        let mut layout_info = vk::PipelineLayoutCreateInfo::default();

        let mut set_layout = vec![];

        let mut idx = 0;
        for descriptor_layout in descriptor_layouts.clone().iter() {
            let set = descriptor_layout.set as usize;
            if idx == set {
                set_layout.push(descriptor_layout.layout);
            } else if idx > set {
                tracing::error!("Wrong order for descriptor sets layouts");
                panic!("Wrong order for descriptor sets layouts");
            } else {
                tracing::info!(target: logger::RENDER, "Gap in pipeline descriptors layout: {}", idx);

                let mut gaps = set - idx;

                while gaps > 0 {
                    let empty_descriptor =
                        DescriptorSetLayout::builder(idx as u32).build(device.clone(), false);
                    set_layout.push(empty_descriptor.layout);
                    descriptor_layouts.insert(idx, empty_descriptor);
                    gaps -= 1;
                    idx += 1;
                }

                set_layout.push(descriptor_layout.layout);
            }

            idx += 1;
        }

        tracing::debug!(target: logger::RENDER, "Create pipline layout with {} descriptor sets", set_layout.len());

        assert_eq!(set_layout.len(), descriptor_layouts.len());

        layout_info = layout_info.set_layouts(&set_layout);

        let mut push_constant_range = vec![];
        if push_constant_size > 0 {
            debug_assert!(push_constant_size <= 128);
            push_constant_range.push(
                vk::PushConstantRange::default()
                    .offset(0)
                    .size(push_constant_size as u32)
                    .stage_flags(vk::ShaderStageFlags::VERTEX),
            );
            layout_info = layout_info.push_constant_ranges(&push_constant_range);
        }

        let layout = unsafe {
            PipelineLayout {
                device: device.clone(),
                _descriptor_layouts: descriptor_layouts,
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
        tracing::debug!(target: logger::MEMORY, "Drop pipeline layout");
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.layout, None);
        }
    }
}
