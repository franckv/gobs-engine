use std::sync::Arc;

use ash::vk;

use crate::buffer::Buffer;
use crate::device::Device;
use crate::image::{Image, ImageLayout, Sampler};
use crate::Wrap;

use super::DescriptorSetLayout;

#[derive(Debug)]
enum ResourceInfo {
    Buffer(vk::DescriptorBufferInfo),
    DynamicBuffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo),
    SampledImage(vk::DescriptorImageInfo),
    ImageCombined(vk::DescriptorImageInfo),
    Sampler(vk::DescriptorImageInfo),
}

/// List of updates to apply on a descriptor set
pub struct DescriptorSetUpdates {
    device: Arc<Device>,
    set: vk::DescriptorSet,
    updates: Vec<ResourceInfo>,
}

impl DescriptorSetUpdates {
    pub fn bind_buffer(mut self, buffer: &Buffer, start: usize, len: usize) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.raw())
            .offset(start as u64)
            .range(len as u64)
            .build();

        self.updates.push(ResourceInfo::Buffer(buffer_info));

        self
    }

    pub fn bind_dynamic_buffer(mut self, buffer: &Buffer, start: usize, len: usize) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.raw())
            .offset(start as u64)
            .range(len as u64)
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

    pub fn bind_sampled_image(mut self, image: &Image, layout: ImageLayout) -> Self {
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(layout.into())
            .image_view(image.image_view)
            .build();

        self.updates.push(ResourceInfo::SampledImage(image_info));

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

    pub fn bind_sampler(mut self, sampler: &Sampler) -> Self {
        let image_info = vk::DescriptorImageInfo::builder()
            .sampler(sampler.raw())
            .build();

        self.updates.push(ResourceInfo::Sampler(image_info));

        self
    }

    pub fn end(self) {
        let mut updates = Vec::new();

        let mut buffer_info_set = vec![];
        let mut image_info_set = vec![];

        for (idx, update) in self.updates.iter().enumerate() {
            let write_info_builder = vk::WriteDescriptorSet::builder()
                .dst_set(self.set)
                .dst_binding(idx as u32)
                .dst_array_element(0)
                .descriptor_type(match update {
                    ResourceInfo::Buffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
                    ResourceInfo::DynamicBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                    ResourceInfo::Image(_) => vk::DescriptorType::STORAGE_IMAGE,
                    ResourceInfo::SampledImage(_) => vk::DescriptorType::SAMPLED_IMAGE,
                    ResourceInfo::ImageCombined(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    ResourceInfo::Sampler(_) => vk::DescriptorType::SAMPLER,
                });

            let write_info = match update {
                ResourceInfo::Buffer(buffer_info) => {
                    buffer_info_set.push(*buffer_info);
                    write_info_builder
                        .buffer_info(std::slice::from_ref(buffer_info_set.last().unwrap()))
                        .build()
                }
                ResourceInfo::DynamicBuffer(buffer_info) => {
                    buffer_info_set.push(*buffer_info);
                    write_info_builder
                        .buffer_info(std::slice::from_ref(buffer_info_set.last().unwrap()))
                        .build()
                }
                ResourceInfo::ImageCombined(image_info) => {
                    image_info_set.push(*image_info);
                    write_info_builder
                        .image_info(std::slice::from_ref(image_info_set.last().unwrap()))
                        .build()
                }
                ResourceInfo::SampledImage(image_info) => {
                    image_info_set.push(*image_info);
                    write_info_builder
                        .image_info(std::slice::from_ref(image_info_set.last().unwrap()))
                        .build()
                }
                ResourceInfo::Image(image_info) => {
                    image_info_set.push(*image_info);
                    write_info_builder
                        .image_info(std::slice::from_ref(image_info_set.last().unwrap()))
                        .build()
                }
                ResourceInfo::Sampler(image_info) => {
                    image_info_set.push(*image_info);
                    write_info_builder
                        .image_info(std::slice::from_ref(image_info_set.last().unwrap()))
                        .build()
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
#[allow(unused)]
pub struct DescriptorSet {
    device: Arc<Device>,
    pub layout: Arc<DescriptorSetLayout>,
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
