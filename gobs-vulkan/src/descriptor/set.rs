use std::sync::Arc;

use ash::vk;

use crate::{
    Wrap,
    buffers::Buffer,
    command::CommandBuffer,
    descriptor::DescriptorSetLayout,
    device::Device,
    images::{Image, ImageLayout, Sampler},
    pipelines::Pipeline,
};

#[derive(Debug)]
struct ResourceInfo {
    binding: u32,
    index: u32,
    ty: ResourceInfoType,
}

#[derive(Debug)]
enum ResourceInfoType {
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

enum DescriptorInfo {
    BufferInfo(vk::DescriptorBufferInfo),
    ImageInfo(vk::DescriptorImageInfo),
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

    pub fn bind_buffer(
        mut self,
        binding: u32,
        index: u32,
        buffer: &Buffer,
        start: u64,
        len: usize,
    ) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer.raw())
            .offset(start)
            .range(len as u64);

        self.updates.push(ResourceInfo {
            binding,
            index,
            ty: ResourceInfoType::Buffer(buffer_info),
        });

        self
    }

    pub fn bind_dynamic_buffer(
        mut self,
        binding: u32,
        index: u32,
        buffer: &Buffer,
        start: usize,
        len: usize,
    ) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer.raw())
            .offset(start as u64)
            .range(len as u64);

        self.updates.push(ResourceInfo {
            binding,
            index,
            ty: ResourceInfoType::DynamicBuffer(buffer_info),
        });

        self
    }

    pub fn bind_image(
        mut self,
        binding: u32,
        index: u32,
        image: &Image,
        layout: ImageLayout,
    ) -> Self {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(layout.into())
            .image_view(image.image_view);

        self.updates.push(ResourceInfo {
            binding,
            index,
            ty: ResourceInfoType::Image(image_info),
        });

        self
    }

    pub fn bind_sampled_image(
        mut self,
        binding: u32,
        index: u32,
        image: &Image,
        layout: ImageLayout,
    ) -> Self {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(layout.into())
            .image_view(image.image_view);

        self.updates.push(ResourceInfo {
            binding,
            index,
            ty: ResourceInfoType::SampledImage(image_info),
        });

        self
    }

    pub fn bind_image_combined(
        mut self,
        binding: u32,
        index: u32,
        image: &Image,
        sampler: &Sampler,
        layout: ImageLayout,
    ) -> Self {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(layout.into())
            .image_view(image.image_view)
            .sampler(sampler.raw());

        self.updates.push(ResourceInfo {
            binding,
            index,
            ty: ResourceInfoType::ImageCombined(image_info),
        });

        self
    }

    pub fn bind_sampler(mut self, binding: u32, index: u32, sampler: &Sampler) -> Self {
        let image_info = vk::DescriptorImageInfo::default().sampler(sampler.raw());

        self.updates.push(ResourceInfo {
            binding,
            index,
            ty: ResourceInfoType::Sampler(image_info),
        });

        self
    }

    fn build_updates<'a>(
        updates: &[ResourceInfo],
        set: Option<&DescriptorSet>,
    ) -> Vec<(vk::WriteDescriptorSet<'a>, DescriptorInfo)> {
        let updates: Vec<(vk::WriteDescriptorSet, DescriptorInfo)> = updates
            .iter()
            .map(|update| {
                let mut write_info = vk::WriteDescriptorSet::default()
                    .dst_binding(update.binding)
                    .dst_array_element(update.index)
                    .descriptor_type(match &update.ty {
                        ResourceInfoType::Buffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
                        ResourceInfoType::DynamicBuffer(_) => {
                            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
                        }
                        ResourceInfoType::Image(_) => vk::DescriptorType::STORAGE_IMAGE,
                        ResourceInfoType::SampledImage(_) => vk::DescriptorType::SAMPLED_IMAGE,
                        ResourceInfoType::ImageCombined(_) => {
                            vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                        }
                        ResourceInfoType::Sampler(_) => vk::DescriptorType::SAMPLER,
                    });

                if let Some(set) = set {
                    write_info = write_info.dst_set(set.raw())
                }

                let resource_info = match &update.ty {
                    ResourceInfoType::Buffer(buffer_info) => {
                        DescriptorInfo::BufferInfo(*buffer_info)
                    }
                    ResourceInfoType::DynamicBuffer(buffer_info) => {
                        DescriptorInfo::BufferInfo(*buffer_info)
                    }
                    ResourceInfoType::ImageCombined(image_info) => {
                        DescriptorInfo::ImageInfo(*image_info)
                    }
                    ResourceInfoType::SampledImage(image_info) => {
                        DescriptorInfo::ImageInfo(*image_info)
                    }
                    ResourceInfoType::Image(image_info) => DescriptorInfo::ImageInfo(*image_info),
                    ResourceInfoType::Sampler(image_info) => DescriptorInfo::ImageInfo(*image_info),
                };

                (write_info, resource_info)
            })
            .collect();

        updates
    }

    fn generate_writes<'a>(
        updates: &'a [(vk::WriteDescriptorSet<'a>, DescriptorInfo)],
    ) -> Vec<vk::WriteDescriptorSet<'a>> {
        updates
            .iter()
            .map(|(write_info, resource_info)| match resource_info {
                DescriptorInfo::BufferInfo(buffer_info) => {
                    write_info.buffer_info(std::slice::from_ref(buffer_info))
                }
                DescriptorInfo::ImageInfo(image_info) => {
                    write_info.image_info(std::slice::from_ref(image_info))
                }
            })
            .collect()
    }

    pub fn write(self, set: &DescriptorSet) {
        let updates = Self::build_updates(&self.updates, Some(set));
        let writes = Self::generate_writes(&updates);

        unsafe {
            self.device.raw().update_descriptor_sets(&writes, &[]);
        }
    }

    pub fn push_descriptors(self, cmd: &CommandBuffer, pipeline: &Pipeline, set: u32) {
        let updates = Self::build_updates(&self.updates, None);
        let writes = Self::generate_writes(&updates);

        unsafe {
            self.device.push_descriptor_device.cmd_push_descriptor_set(
                cmd.raw(),
                pipeline.bind_point,
                pipeline.layout.layout,
                set,
                &writes,
            );
        }
    }
}
