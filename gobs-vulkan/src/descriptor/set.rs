use std::mem;
use std::ptr;
use std::sync::Arc;

use ash::vk;

use crate::buffer::Buffer;
use crate::device::Device;
use crate::image::{Image, Sampler};
use crate::Wrap;

enum ResourceInfo {
    Buffer(vk::DescriptorBufferInfo),
    DynamicBuffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo),
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

        let buffer_info = vk::DescriptorBufferInfo {
            buffer: buffer.raw(),
            offset: (start * item_size) as u64,
            range: (len * item_size) as u64,
        };

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

        let buffer_info = vk::DescriptorBufferInfo {
            buffer: buffer.raw(),
            offset: (start * item_size) as u64,
            range: (len * item_size) as u64,
        };

        self.updates.push(ResourceInfo::DynamicBuffer(buffer_info));

        self
    }

    pub fn bind_image(mut self, image: &Image, sampler: &Sampler) -> Self {
        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: image.image_view,
            sampler: sampler.raw(),
        };

        self.updates.push(ResourceInfo::Image(image_info));

        self
    }

    pub fn end(self) {
        let mut updates = Vec::new();

        for (idx, update) in self.updates.iter().enumerate() {
            updates.push(vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: self.set,
                dst_binding: idx as u32,
                dst_array_element: 0,
                descriptor_type: match update {
                    ResourceInfo::Buffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
                    ResourceInfo::DynamicBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                    ResourceInfo::Image(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                },
                descriptor_count: 1,
                p_buffer_info: match update {
                    ResourceInfo::Buffer(buffer_info) => buffer_info,
                    ResourceInfo::DynamicBuffer(buffer_info) => buffer_info,
                    ResourceInfo::Image(_) => ptr::null(),
                },
                p_image_info: match update {
                    ResourceInfo::Buffer(_) => ptr::null(),
                    ResourceInfo::DynamicBuffer(_) => ptr::null(),
                    ResourceInfo::Image(image_info) => image_info,
                },
                p_texel_buffer_view: ptr::null(),
            });
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
    set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub(crate) fn new(device: Arc<Device>, set: vk::DescriptorSet) -> Self {
        DescriptorSet { device, set }
    }

    pub fn start_update(&self) -> DescriptorSetUpdates {
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
