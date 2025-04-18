use std::sync::Arc;

use ash::vk;

use crate::{
    Wrap,
    buffer::Buffer,
    command::CommandBuffer,
    descriptor::DescriptorSetLayout,
    device::Device,
    image::{Image, ImageLayout, Sampler},
    pipeline::Pipeline,
};

#[derive(Debug)]
enum ResourceInfo {
    Buffer(vk::DescriptorBufferInfo),
    DynamicBuffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo),
    SampledImage(vk::DescriptorImageInfo),
    ImageCombined(vk::DescriptorImageInfo),
    Sampler(vk::DescriptorImageInfo),
}

/// Bind resources to shaders
#[derive(Clone, Debug)]
pub struct DescriptorSet {
    pub device: Arc<Device>,
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
}

impl Wrap<vk::DescriptorSet> for DescriptorSet {
    fn raw(&self) -> vk::DescriptorSet {
        self.set
    }
}

/// List of updates to apply on a descriptor set
pub struct DescriptorSetUpdates {
    device: Arc<Device>,
    updates: Vec<ResourceInfo>,
}

impl DescriptorSetUpdates {
    pub fn new(device: Arc<Device>) -> Self {
        DescriptorSetUpdates {
            device: device.clone(),
            updates: Vec::new(),
        }
    }

    pub fn bind_buffer(mut self, buffer: &Buffer, start: usize, len: usize) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer.raw())
            .offset(start as u64)
            .range(len as u64);

        self.updates.push(ResourceInfo::Buffer(buffer_info));

        self
    }

    pub fn bind_dynamic_buffer(mut self, buffer: &Buffer, start: usize, len: usize) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer.raw())
            .offset(start as u64)
            .range(len as u64);

        self.updates.push(ResourceInfo::DynamicBuffer(buffer_info));

        self
    }

    pub fn bind_image(mut self, image: &Image, layout: ImageLayout) -> Self {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(layout.into())
            .image_view(image.image_view);

        self.updates.push(ResourceInfo::Image(image_info));

        self
    }

    pub fn bind_sampled_image(mut self, image: &Image, layout: ImageLayout) -> Self {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(layout.into())
            .image_view(image.image_view);

        self.updates.push(ResourceInfo::SampledImage(image_info));

        self
    }

    pub fn bind_image_combined(
        mut self,
        image: &Image,
        sampler: &Sampler,
        layout: ImageLayout,
    ) -> Self {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(layout.into())
            .image_view(image.image_view)
            .sampler(sampler.raw());

        self.updates.push(ResourceInfo::ImageCombined(image_info));

        self
    }

    pub fn bind_sampler(mut self, sampler: &Sampler) -> Self {
        let image_info = vk::DescriptorImageInfo::default().sampler(sampler.raw());

        self.updates.push(ResourceInfo::Sampler(image_info));

        self
    }

    pub fn write(self, set: &DescriptorSet) {
        let mut updates = Vec::new();

        let mut buffer_info_set = vec![];
        let mut image_info_set = vec![];

        for (idx, update) in self.updates.iter().enumerate() {
            let write_info = vk::WriteDescriptorSet::default()
                .dst_set(set.raw())
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

            match update {
                ResourceInfo::Buffer(buffer_info) => {
                    buffer_info_set.push((write_info, vec![*buffer_info]));
                }
                ResourceInfo::DynamicBuffer(buffer_info) => {
                    buffer_info_set.push((write_info, vec![*buffer_info]));
                }
                ResourceInfo::ImageCombined(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
                ResourceInfo::SampledImage(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
                ResourceInfo::Image(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
                ResourceInfo::Sampler(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
            };
        }

        for (write_info, buffer_info) in &buffer_info_set {
            let write_info = write_info.buffer_info(buffer_info);
            updates.push(write_info);
        }

        for (write_info, image_info) in &image_info_set {
            let write_info = write_info.image_info(image_info);
            updates.push(write_info);
        }

        unsafe {
            self.device
                .raw()
                .update_descriptor_sets(updates.as_ref(), &[]);
        }
    }

    pub fn push_descriptors(self, cmd: &CommandBuffer, pipeline: &Pipeline, set: u32) {
        let mut updates = Vec::new();

        let mut buffer_info_set = vec![];
        let mut image_info_set = vec![];

        for (idx, update) in self.updates.iter().enumerate() {
            let write_info = vk::WriteDescriptorSet::default()
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

            match update {
                ResourceInfo::Buffer(buffer_info) => {
                    buffer_info_set.push((write_info, vec![*buffer_info]));
                }
                ResourceInfo::DynamicBuffer(buffer_info) => {
                    buffer_info_set.push((write_info, vec![*buffer_info]));
                }
                ResourceInfo::ImageCombined(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
                ResourceInfo::SampledImage(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
                ResourceInfo::Image(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
                ResourceInfo::Sampler(image_info) => {
                    image_info_set.push((write_info, vec![*image_info]));
                }
            };
        }

        for (write_info, buffer_info) in &buffer_info_set {
            let write_info = write_info.buffer_info(buffer_info);
            updates.push(write_info);
        }

        for (write_info, image_info) in &image_info_set {
            let write_info = write_info.image_info(image_info);
            updates.push(write_info);
        }

        unsafe {
            self.device.push_descriptor_device.cmd_push_descriptor_set(
                cmd.raw(),
                pipeline.bind_point,
                pipeline.layout.layout,
                set,
                updates.as_ref(),
            );
        }
    }
}
