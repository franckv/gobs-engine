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

pub(crate) struct BindingRegistry {
    frames_in_flight: usize,
    pools: Vec<HashMap<BindingGroupLayout, DescriptorSetPool>>,
}

const MAX_SET: usize = 10;

impl BindingRegistry {
    pub fn new(frames_in_flight: usize) -> Self {
        Self {
            frames_in_flight,
            pools: (0..frames_in_flight).map(|_| HashMap::new()).collect(),
        }
    }

    pub fn reset(&mut self, frame_id: usize) {
        let mut map = &mut self.pools[frame_id];

        for pool in map.values_mut() {
            pool.reset();
        }
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

    fn get_pool(
        &mut self,
        device: Arc<vk::Device>,
        resource: &BindResource,
        frame_id: usize,
    ) -> &mut DescriptorSetPool {
        let mut map = &mut self.pools[frame_id];

        map.entry(resource.layout.clone()).or_insert_with(|| {
            DescriptorSetPool::new(
                device.clone(),
                vk_layout(device.clone(), &resource.layout),
                MAX_SET,
            )
        })
    }

    pub fn get_ds(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
        frame_id: usize,
    ) -> DescriptorSet {
        let ds_pool = self.get_pool(device.clone(), resource, frame_id);

        let ds = ds_pool.allocate();

        let update = self.generate_update(device, registry, resource);

        update.write(&ds);

        ds
    }

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
