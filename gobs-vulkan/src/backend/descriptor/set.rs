use std::mem;
use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::backend::buffer::Buffer;
use crate::backend::device::Device;
use crate::backend::image::{Image, Sampler};
use crate::backend::Wrap;

enum ResourceInfo {
    Buffer(vk::DescriptorBufferInfo),
    DynamicBuffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo)
}

pub struct DescriptorSetResources {
    device: Arc<Device>,
    set: vk::DescriptorSet,
    infos: Vec<ResourceInfo>
}

impl DescriptorSetResources {
    pub fn new(set: &mut DescriptorSet) -> Self {
        DescriptorSetResources {
            device: set.device.clone(),
            set: set.raw(),
            infos: Vec::new(),
        }
    }

    pub fn bind_buffer<T: Copy>(mut self, buffer: &Buffer<T>,
                                start: usize, len: usize) -> Self {
        let item_size = mem::size_of::<T>();

        self.infos.push(ResourceInfo::Buffer(vk::DescriptorBufferInfo {
            buffer: buffer.raw(),
            offset: (start * item_size) as u64,
            range: (len * item_size) as u64,
        }));

        self
    }

    pub fn bind_dynamic_buffer<T: Copy>(mut self, buffer: &Buffer<T>,
                                start: usize, len: usize) -> Self {
        let item_size = mem::size_of::<T>();

        self.infos.push(ResourceInfo::DynamicBuffer(vk::DescriptorBufferInfo {
            buffer: buffer.raw(),
            offset: (start * item_size) as u64,
            range: (len * item_size) as u64,
        }));

        self
    }

    pub fn bind_image(mut self, image: &Image, sampler: &Sampler) -> Self {
        self.infos.push(ResourceInfo::Image(vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: image.image_view,
            sampler: sampler.raw(),
        }));

        self
    }

    pub fn update(self) {
        let mut updates = Vec::new();

        for (idx, info) in self.infos.iter().enumerate() {
            match info {
                ResourceInfo::Buffer(buffer_info) =>
                    updates.push(vk::WriteDescriptorSet {
                        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                        p_next: ptr::null(),
                        dst_set: self.set,
                        dst_binding: idx as u32,
                        dst_array_element: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        p_buffer_info: buffer_info,
                        p_image_info: ptr::null(),
                        p_texel_buffer_view: ptr::null(),
                    }),
                ResourceInfo::DynamicBuffer(buffer_info) =>
                    updates.push(vk::WriteDescriptorSet {
                        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                        p_next: ptr::null(),
                        dst_set: self.set,
                        dst_binding: idx as u32,
                        dst_array_element: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                        descriptor_count: 1,
                        p_buffer_info: buffer_info,
                        p_image_info: ptr::null(),
                        p_texel_buffer_view: ptr::null(),
                    }),
                ResourceInfo::Image(image_info) =>
                    updates.push(vk::WriteDescriptorSet {
                        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                        p_next: ptr::null(),
                        dst_set: self.set,
                        dst_binding: idx as u32,
                        dst_array_element: 0,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1,
                        p_buffer_info: ptr::null(),
                        p_image_info: image_info,
                        p_texel_buffer_view: ptr::null(),
                    })
            }
        }
        unsafe {
            self.device.raw().update_descriptor_sets(updates.as_ref(),
                                                     &[]);
        }
    }
}

pub struct DescriptorSet {
    device: Arc<Device>,
    set: vk::DescriptorSet
}

impl DescriptorSet {
    pub(crate) fn new(device: Arc<Device>, set: vk::DescriptorSet) -> Self {
        DescriptorSet {
            device,
            set,
        }
    }
}

impl Wrap<vk::DescriptorSet> for DescriptorSet {
    fn raw(&self) -> vk::DescriptorSet {
        self.set
    }
}
