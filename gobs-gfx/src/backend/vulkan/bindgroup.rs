use std::sync::Arc;

use gobs_vulkan as vk;

use crate::backend::vulkan::{
    buffer::VkBuffer, device::VkDevice, image::VkImage, image::VkSampler, renderer::VkRenderer,
};
use crate::bindgroup::{BindingGroupLayout, BindingGroupPool};
use crate::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, DescriptorStage, DescriptorType,
    ImageLayout,
};

#[derive(Clone, Debug)]
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

impl VkBindingGroupLayout {
    pub(crate) fn vk_layout(
        &self,
        device: Arc<VkDevice>,
    ) -> Arc<vk::descriptor::DescriptorSetLayout> {
        let mut ds_layout =
            vk::descriptor::DescriptorSetLayout::builder(self.binding_group_type.set());

        for (ty, stage) in &self.bindings {
            ds_layout = ds_layout.binding(*ty, *stage);
        }

        ds_layout.build(device.device.clone(), self.binding_group_type.is_push())
    }
}

#[derive(Debug)]
pub struct VkBindingGroupPool {
    pub(crate) layout: VkBindingGroupLayout,
    pool: vk::descriptor::DescriptorSetPool,
}

impl VkBindingGroupPool {
    pub fn new(device: Arc<VkDevice>, pool_size: usize, layout: VkBindingGroupLayout) -> Self {
        let ds_layout = layout.vk_layout(device.clone());

        let pool =
            vk::descriptor::DescriptorSetPool::new(device.device.clone(), ds_layout, pool_size);

        Self { pool, layout }
    }
}

impl BindingGroupPool<VkRenderer> for VkBindingGroupPool {
    fn allocate(&mut self) -> VkBindingGroup {
        let ds = self.pool.allocate();

        VkBindingGroup {
            ds,
            bind_group_type: self.layout.binding_group_type,
        }
    }

    fn reset(&mut self) {
        self.pool.reset();
    }
}

#[derive(Clone, Debug)]
pub struct VkBindingGroup {
    pub ds: vk::descriptor::DescriptorSet,
    pub bind_group_type: BindingGroupType,
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
