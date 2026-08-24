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
    backend::vulkan::registry::ResourcesRegistry, bindings::BindingLifetime,
};

pub(crate) struct BindingRegistry {
    frames_in_flight: usize,
    pub(crate) per_frame_pools: Vec<HashMap<u64, DescriptorSetPool>>,
    pub(crate) per_frame_ds_cache: Vec<HashMap<BindingId, DescriptorSet>>,
    pub(crate) static_pools: HashMap<u64, DescriptorSetPool>,
    pub(crate) static_ds_cache: HashMap<BindingId, DescriptorSet>,
}

const MAX_SET: usize = 10;

impl BindingRegistry {
    pub fn new(frames_in_flight: usize) -> Self {
        Self {
            frames_in_flight,
            per_frame_pools: (0..frames_in_flight).map(|_| HashMap::new()).collect(),
            per_frame_ds_cache: (0..frames_in_flight).map(|_| HashMap::new()).collect(),
            static_pools: HashMap::new(),
            static_ds_cache: HashMap::new(),
        }
    }

    pub fn reset(&mut self, frame_id: usize) {
        let mut pool_map = &mut self.per_frame_pools[frame_id];

        for pool in pool_map.values_mut() {
            pool.reset();
        }

        let mut ds_cache = &mut self.per_frame_ds_cache[frame_id];
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

        update.push_descriptors(cmd, pipeline, resource.layout().binding_group_type.set());
    }

    fn get_pool(
        &mut self,
        device: Arc<vk::Device>,
        resource: &BindResource,
        frame_id: usize,
        lifetime: BindingLifetime,
    ) -> &mut DescriptorSetPool {
        let (map, max_sets) = match lifetime {
            BindingLifetime::PerFrame => (&mut self.per_frame_pools[frame_id], MAX_SET),
            BindingLifetime::Static => (&mut self.static_pools, 1),
        };

        map.entry(resource.layout().binding_group_id)
            .or_insert_with(|| {
                DescriptorSetPool::new(
                    device.clone(),
                    vk_layout(device.clone(), resource.layout()),
                    max_sets,
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
        let binding_type = resource.layout().binding_group_type;
        let lifetime = binding_type.lifetime();

        let cache = match lifetime {
            BindingLifetime::PerFrame => &mut self.per_frame_ds_cache[frame_id],
            BindingLifetime::Static => &mut self.static_ds_cache,
        };

        if let Some(ds) = cache.get(&resource.id) {
            return ds.clone();
        }

        let ds_pool = self.get_pool(device.clone(), resource, frame_id, lifetime);

        let ds = ds_pool.allocate();

        let update = self.generate_update(device, registry, resource);

        update.write(&ds);

        let cache = match lifetime {
            BindingLifetime::PerFrame => &mut self.per_frame_ds_cache[frame_id],
            BindingLifetime::Static => &mut self.static_ds_cache,
        };

        cache.insert(resource.id, ds.clone());

        ds
    }

    pub fn update_static_ds(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
        binding: usize,
        index: usize,
    ) {
        debug_assert!(resource.layout().bindings.len() > binding);
        debug_assert_eq!(resource.layout().bindings.len(), resource.sets());

        tracing::debug!("Update static ds, binding={}, index={}", binding, index);

        let ds = self.get_ds(device.clone(), registry, resource, 0);

        let mut update = DescriptorSetUpdates::new(device);

        let (ty, _, count) = resource.layout().bindings[binding];
        let bindset = resource.bindset(binding);

        let binding_idx: u32 = resource.layout().bindings[..binding]
            .iter()
            .map(|(_, _, count)| count)
            .sum();

        debug_assert!(count > index as u32);

        let handle = bindset.get(index).unwrap_or_else(|| {
            panic!(
                "Invalid binding {} with index: {} for ty={:?}",
                binding, index, ty
            )
        });

        match ty {
            vk::DescriptorType::SampledImage => {
                if let Some(image) = registry.images.get(handle) {
                    update = update.bind_sampled_image(
                        binding_idx,
                        index as u32,
                        image,
                        ImageLayout::Shader,
                    );
                }
            }
            _ => todo!(),
        }

        update.write(&ds);
    }

    fn generate_update(
        &mut self,
        device: Arc<vk::Device>,
        registry: &ResourcesRegistry,
        resource: &BindResource,
    ) -> DescriptorSetUpdates {
        let mut update = DescriptorSetUpdates::new(device);

        debug_assert_eq!(resource.layout().bindings.len(), resource.sets());

        let mut binding_idx = 0;

        for ((ty, stage, count), bindsets) in
            resource.layout().bindings.iter().zip(resource.bindsets())
        {
            debug_assert!(bindsets.len() <= *count as usize);
            for (handle, index) in bindsets.bindings() {
                match ty {
                    // scene data, material data
                    vk::DescriptorType::Uniform => {
                        if let Some(buffer) = registry.buffers.get(*handle) {
                            update = update.bind_uniform_buffer(
                                binding_idx,
                                *index as u32,
                                &buffer.buffer,
                                buffer.offset,
                                buffer.len,
                            );
                        }
                    }
                    // material ssbo
                    vk::DescriptorType::StorageBuffer => {
                        if let Some(buffer) = registry.buffers.get(*handle) {
                            update = update.bind_storage_buffer(
                                binding_idx,
                                *index as u32,
                                &buffer.buffer,
                                buffer.offset,
                                buffer.len,
                            );
                        }
                    }
                    // compute data
                    vk::DescriptorType::StorageImage => {
                        if let Some(image) = registry.images.get(*handle) {
                            update = update.bind_image(
                                binding_idx,
                                *index as u32,
                                image,
                                ImageLayout::General,
                            );
                        }
                    }
                    // texture
                    vk::DescriptorType::Sampler => {
                        if let Some(sampler) = registry.samplers.get(*handle) {
                            update = update.bind_sampler(binding_idx, *index as u32, sampler);
                        }
                    }
                    vk::DescriptorType::SampledImage => {
                        if let Some(image) = registry.images.get(*handle) {
                            update = update.bind_sampled_image(
                                binding_idx,
                                *index as u32,
                                image,
                                ImageLayout::Shader,
                            );
                        }
                    }
                    // unused
                    _ => todo!(),
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
