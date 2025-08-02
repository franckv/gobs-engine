use std::sync::Arc;

use gobs_vulkan as vk;
use gobs_vulkan::descriptor::DescriptorSetLayout;

use crate::backend::vulkan::{
    buffer::VkBuffer, device::VkDevice, image::VkImage, image::VkSampler, pipeline::VkPipeline,
    renderer::VkRenderer,
};
use crate::bindgroup::{BindingGroupLayout, BindingGroupPool};
use crate::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, DescriptorStage, DescriptorType,
    ImageLayout,
};

#[derive(Debug)]
pub struct VkBindingGroupLayout {
    pub binding_group_type: BindingGroupType,
    pub bindings: Vec<(DescriptorType, DescriptorStage)>,
}

impl BindingGroupLayout<VkRenderer> for VkBindingGroupLayout {
    fn new(binding_group_type: BindingGroupType) -> Self {
        Self {
            binding_group_type,
            bindings: Vec::new(),
        }
    }

    fn add_binding(mut self, ty: DescriptorType, stage: DescriptorStage) -> Self {
        self.bindings.push((ty, stage));

        self
    }
}

#[derive(Debug)]
pub struct VkBindingGroupPool {
    pub(crate) bind_group_type: BindingGroupType,
    pub(crate) layout: Arc<DescriptorSetLayout>,
    pool: vk::descriptor::DescriptorSetPool,
}

impl VkBindingGroupPool {
    pub fn new(
        device: Arc<VkDevice>,
        bind_group_type: BindingGroupType,
        pool_size: usize,
        layout: Arc<DescriptorSetLayout>,
    ) -> Self {
        let pool = vk::descriptor::DescriptorSetPool::new(
            device.device.clone(),
            layout.clone(),
            pool_size,
        );

        Self {
            bind_group_type,
            pool,
            layout,
        }
    }
}

impl BindingGroupPool<VkRenderer> for VkBindingGroupPool {
    fn allocate(&mut self, pipeline: Arc<VkPipeline>) -> VkBindingGroup {
        let ds = self.pool.allocate();

        VkBindingGroup {
            ds,
            bind_group_type: self.bind_group_type,
            pipeline,
        }
    }

    fn reset(&mut self) {
        self.pool.reset();
    }
}

#[derive(Clone, Debug)]
pub struct VkBindingGroup {
    pub(crate) ds: vk::descriptor::DescriptorSet,
    pub(crate) bind_group_type: BindingGroupType,
    pub(crate) pipeline: Arc<VkPipeline>,
}

impl BindingGroup<VkRenderer> for VkBindingGroup {
    fn update(&self) -> VkBindingGroupUpdates {
        VkBindingGroupUpdates {
            set: self.ds.clone(),
            update: vk::descriptor::DescriptorSetUpdates::new(self.ds.device.clone()),
        }
    }
}

pub struct VkBindingGroupUpdates {
    set: vk::descriptor::DescriptorSet,
    update: vk::descriptor::DescriptorSetUpdates,
}

impl BindingGroupUpdates<VkRenderer> for VkBindingGroupUpdates {
    fn bind_buffer(mut self, buffer: &VkBuffer, start: usize, len: usize) -> Self {
        self.update = self.update.bind_buffer(&buffer.buffer, start, len);

        self
    }

    fn bind_image(mut self, image: &VkImage, layout: ImageLayout) -> Self {
        self.update = self.update.bind_image(&image.image, layout);

        self
    }

    fn bind_sampled_image(mut self, image: &VkImage, layout: ImageLayout) -> Self {
        self.update = self.update.bind_sampled_image(&image.image, layout);

        self
    }

    fn bind_sampler(mut self, sampler: &VkSampler) -> Self {
        self.update = self.update.bind_sampler(&sampler.sampler);

        self
    }

    fn end(self) {
        self.update.write(&self.set);
    }
}
