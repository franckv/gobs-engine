use gobs_vulkan::{DescriptorStage, DescriptorType};

use crate::{BindResource, BindingGroupLayout, BindingGroupType, BufferType, RenderHAL};

pub struct MaterialRegistry {
    free_list: Vec<usize>,
    binding: BindResource,
}

impl MaterialRegistry {
    pub fn new(hal: &mut dyn RenderHAL, size: usize) -> Self {
        let free_list = (0..size).rev().collect();

        let layout = BindingGroupLayout::new(BindingGroupType::BindlessMaterial).add_binding(
            DescriptorType::StorageBuffer,
            DescriptorStage::Fragment,
            1,
        );

        let buffer = hal.create_buffer("Bindless material", size, BufferType::Storage);

        let binding = BindResource::new(layout).binding(buffer, 0);

        Self { free_list, binding }
    }
}
