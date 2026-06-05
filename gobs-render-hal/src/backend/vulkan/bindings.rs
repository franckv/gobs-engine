use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use gobs_vulkan::{self as vk, Pipeline};

use vk::descriptor::{DescriptorSet, DescriptorSetPool, DescriptorSetUpdates};
use vk::images::ImageLayout;

use crate::{
    BindResource, BindingGroupLayout, Handle, backend::vulkan::registry::ResourcesRegistry,
};

#[derive(Default)]
pub(crate) struct BindingRegistry {
    pools: HashMap<BindingGroupLayout, DescriptorSetPool>,
}

impl BindingRegistry {
    fn generate_update(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
    ) -> DescriptorSetUpdates {
        let mut update = DescriptorSetUpdates::new(device);

        let BindResource {
            layout:
                BindingGroupLayout {
                    binding_group_type,
                    bindings,
                },
            resources,
        } = resource;

        debug_assert_eq!(resources.len(), bindings.len());

        // TODO: bind descriptor set
        for ((ty, stage), handle) in bindings.iter().zip(resources) {
            match ty {
                // scene data, material data
                vk::DescriptorType::Uniform => {
                    if let Some(buffer) = registry.buffers.get(*handle) {
                        update = update.bind_buffer(&buffer.buffer, buffer.offset, buffer.len);
                    }
                }
                // compute data
                vk::DescriptorType::StorageImage => {
                    if let Some(image) = registry.images.get(*handle) {
                        // TODO: hardcoded
                        update = update.bind_image(image, ImageLayout::General);
                    }
                }
                // texture
                vk::DescriptorType::Sampler => {
                    if let Some(sampler) = registry.samplers.get(*handle) {
                        update = update.bind_sampler(sampler);
                    }
                }
                vk::DescriptorType::SampledImage => {
                    if let Some(image) = registry.images.get(*handle) {
                        update = update.bind_sampled_image(image, ImageLayout::Shader);
                    }
                }
                // unused
                vk::DescriptorType::UniformDynamic => todo!(),
                vk::DescriptorType::ImageSampler => todo!(),
            }
        }

        update
    }

    pub fn push_descriptor(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
        pipeline: &Pipeline,
        cmd: &vk::CommandBuffer,
    ) {
        let update = self.generate_update(device, registry, resource);

        update.push_descriptors(cmd, pipeline, resource.layout.binding_group_type.set());
    }

    pub fn allocate_ds(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
    ) -> DescriptorSet {
        let ds = match self.pools.entry(resource.layout.clone()) {
            Entry::Occupied(mut e) => e.get_mut().allocate(),
            Entry::Vacant(e) => e
                .insert(DescriptorSetPool::new(
                    device.clone(),
                    vk_layout(device.clone(), &resource.layout),
                    10,
                ))
                .allocate(),
        };

        let update = self.generate_update(device, registry, resource);

        update.write(&ds);

        ds
    }
}

pub(crate) fn vk_layout(
    device: Arc<vk::Device>,
    layout: &BindingGroupLayout,
) -> Arc<vk::descriptor::DescriptorSetLayout> {
    let mut ds_layout =
        vk::descriptor::DescriptorSetLayout::builder(layout.binding_group_type.set());

    for (ty, stage) in &layout.bindings {
        ds_layout = ds_layout.binding(*ty, *stage);
    }

    ds_layout.build(device.clone(), layout.binding_group_type.is_push())
}
