use std::sync::Arc;

use gobs_vulkan as vk;

use crate::backend::vulkan::{
    buffer::VkBuffer, image::VkImage, image::VkSampler, pipeline::VkPipeline, renderer::VkRenderer,
};
use crate::{BindingGroup, BindingGroupType, BindingGroupUpdates, ImageLayout};

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
