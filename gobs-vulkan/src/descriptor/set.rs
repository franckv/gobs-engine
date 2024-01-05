use std::mem;
use std::sync::Arc;

use ash::vk;

use crate::buffer::Buffer;
use crate::device::Device;
use crate::image::{Image, ImageLayout, Sampler};
use crate::Wrap;

use super::DescriptorSetLayout;

enum ResourceInfo {
    Buffer(vk::DescriptorBufferInfo),
    DynamicBuffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo),
    ImageCombined(vk::DescriptorImageInfo),
}

/// List of updates to apply on a descriptor set
pub struct DescriptorSetUpdates {
    device: Arc<Device>,
    set: vk::DescriptorSet,
    updates: Vec<ResourceInfo>,
}

impl DescriptorSetUpdates {
    pub fn bind_buffer<T: Copy>(mut self, buffer: &Buffer<T>, start: usize, len: usize) -> Self {
        let item_size = mem::size_of::<T>();

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.raw())
            .offset((start * item_size) as u64)
            .range((len * item_size) as u64)
            .build();

        self.updates.push(ResourceInfo::Buffer(buffer_info));

        self
    }

    pub fn bind_dynamic_buffer<T: Copy>(
        mut self,
        buffer: &Buffer<T>,
        start: usize,
        len: usize,
    ) -> Self {
        let item_size = mem::size_of::<T>();

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.raw())
            .offset((start * item_size) as u64)
            .range((len * item_size) as u64)
            .build();

        self.updates.push(ResourceInfo::DynamicBuffer(buffer_info));

        self
    }

    pub fn bind_image(mut self, image: &Image, layout: ImageLayout) -> Self {
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(layout.into())
            .image_view(image.image_view)
            .build();

        self.updates.push(ResourceInfo::Image(image_info));

        self
    }

    pub fn bind_image_combined(
        mut self,
        image: &Image,
        sampler: &Sampler,
        layout: ImageLayout,
    ) -> Self {
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(layout.into())
            .image_view(image.image_view)
            .sampler(sampler.raw())
            .build();

        self.updates.push(ResourceInfo::ImageCombined(image_info));

        self
    }

    pub fn end(self) {
        let mut updates = Vec::new();

        for (idx, update) in self.updates.iter().enumerate() {
            let write_info_builder = vk::WriteDescriptorSet::builder()
                .dst_set(self.set)
                .dst_binding(idx as u32)
                .dst_array_element(0)
                .descriptor_type(match update {
                    ResourceInfo::Buffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
                    ResourceInfo::DynamicBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                    ResourceInfo::Image(_) => vk::DescriptorType::STORAGE_IMAGE,
                    ResourceInfo::ImageCombined(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                });

            let write_info = match update {
                ResourceInfo::Buffer(buffer_info) => {
                    write_info_builder.buffer_info(&[*buffer_info]).build()
                }
                ResourceInfo::DynamicBuffer(buffer_info) => {
                    write_info_builder.buffer_info(&[*buffer_info]).build()
                }
                ResourceInfo::ImageCombined(image_info) => {
                    write_info_builder.image_info(&[*image_info]).build()
                }
                ResourceInfo::Image(image_info) => {
                    write_info_builder.image_info(&[*image_info]).build()
                }
            };

            updates.push(write_info);
        }

        unsafe {
            self.device
                .raw()
                .update_descriptor_sets(updates.as_ref(), &[]);
        }
    }
}

/// Bind resources to shaders
pub struct DescriptorSet {
    device: Arc<Device>,
    layout: Arc<DescriptorSetLayout>,
    set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub(crate) fn new(
        device: Arc<Device>,
        set: vk::DescriptorSet,
        layout: Arc<DescriptorSetLayout>,
    ) -> Self {
        DescriptorSet {
            device,
            layout,
            set,
        }
    }

    pub fn update(&self) -> DescriptorSetUpdates {
        DescriptorSetUpdates {
            device: self.device.clone(),
            set: self.raw(),
            updates: Vec::new(),
        }
    }
}

impl Wrap<vk::DescriptorSet> for DescriptorSet {
    fn raw(&self) -> vk::DescriptorSet {
        self.set
    }
}
