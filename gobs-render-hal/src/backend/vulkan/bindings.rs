use std::{collections::hash_map::Entry, sync::Arc};

use ahash::{HashMap, HashMapExt as _};

use gobs_vulkan as vk;

use vk::{
    Pipeline,
    descriptor::{DescriptorSet, DescriptorSetPool, DescriptorSetUpdates},
    images::ImageLayout,
};

use crate::{
    BindResource, BindingGroupLayout, BindingId, Handle,
    backend::vulkan::registry::ResourcesRegistry,
};

pub(crate) struct BindingRegistry {
    frames_in_flight: usize,
    pub(crate) pools: Vec<HashMap<u64, DescriptorSetPool>>,
    pub(crate) ds_cache: Vec<HashMap<BindingId, DescriptorSet>>,
}

const MAX_SET: usize = 10;

impl BindingRegistry {
    pub fn new(frames_in_flight: usize) -> Self {
        Self {
            frames_in_flight,
            pools: (0..frames_in_flight).map(|_| HashMap::new()).collect(),
            ds_cache: (0..frames_in_flight).map(|_| HashMap::new()).collect(),
        }
    }

    pub fn reset(&mut self, frame_id: usize) {
        let mut pool_map = &mut self.pools[frame_id];

        for pool in pool_map.values_mut() {
            pool.reset();
        }

        let mut ds_cache = &mut self.ds_cache[frame_id];
        ds_cache.clear();
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

        map.entry(resource.layout.binding_group_id)
            .or_insert_with(|| {
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
        if let Some(ds) = self.ds_cache[frame_id].get(&resource.id) {
            ds.clone()
        } else {
            let ds_pool = self.get_pool(device.clone(), resource, frame_id);

            let ds = ds_pool.allocate();

            let update = self.generate_update(device, registry, resource);

            update.write(&ds);

            self.ds_cache[frame_id].insert(resource.id, ds.clone());

            ds
        }
    }

    fn generate_update(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
    ) -> DescriptorSetUpdates {
        let mut update = DescriptorSetUpdates::new(device);

        let BindResource {
            id,
            layout:
                BindingGroupLayout {
                    binding_group_id,
                    binding_group_type,
                    bindings,
                },
            resources,
        } = resource;

        let n_bindings: usize = bindings.iter().map(|(_, _, count)| *count as usize).sum();

        debug_assert_eq!(resources.len(), n_bindings);

        let mut binding_idx = 0;
        let mut idx = 0;

        for (ty, stage, count) in bindings {
            for descriptor in 0..*count {
                let handle = resources[idx];
                idx += 1;

                match ty {
                    // scene data, material data
                    vk::DescriptorType::Uniform => {
                        if let Some(buffer) = registry.buffers.get(handle) {
                            update = update.bind_buffer(
                                binding_idx,
                                descriptor,
                                &buffer.buffer,
                                buffer.offset,
                                buffer.len,
                            );
                        }
                    }
                    // compute data
                    vk::DescriptorType::StorageImage => {
                        if let Some(image) = registry.images.get(handle) {
                            update = update.bind_image(
                                binding_idx,
                                descriptor,
                                image,
                                ImageLayout::General,
                            );
                        }
                    }
                    // texture
                    vk::DescriptorType::Sampler => {
                        if let Some(sampler) = registry.samplers.get(handle) {
                            update = update.bind_sampler(binding_idx, descriptor, sampler);
                        }
                    }
                    vk::DescriptorType::SampledImage => {
                        if let Some(image) = registry.images.get(handle) {
                            update = update.bind_sampled_image(
                                binding_idx,
                                descriptor,
                                image,
                                ImageLayout::Shader,
                            );
                        }
                    }
                    // unused
                    vk::DescriptorType::UniformDynamic => todo!(),
                    vk::DescriptorType::ImageSampler => todo!(),
                }
            }

            binding_idx += count;
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

    for (ty, stage, count) in &layout.bindings {
        ds_layout = ds_layout.binding(*ty, *stage, *count);
    }

    ds_layout.build(device.clone(), layout.binding_group_type.is_push())
}
