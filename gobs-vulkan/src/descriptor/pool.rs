use std::ptr;
use std::sync::Arc;

use ash::vk;

use log::trace;

use crate::descriptor::{DescriptorSet, DescriptorSetLayout};
use crate::device::Device;
use crate::Wrap;

pub struct DescriptorSetPool {
    device: Arc<Device>,
    pool: vk::DescriptorPool,
    sets: Vec<DescriptorSet>,
    current: usize
}

impl DescriptorSetPool {
    pub fn new(device: Arc<Device>,
               descriptor_layout: Arc<DescriptorSetLayout>,
               count: usize) -> Self {
        let pool_size: Vec<vk::DescriptorPoolSize> =
            descriptor_layout.bindings.iter().map(|binding| {
                vk::DescriptorPoolSize {
                    ty: binding.descriptor_type,
                    descriptor_count: count as u32,
                }
            }).collect();

        let pool_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            pool_size_count: pool_size.len() as u32,
            p_pool_sizes: pool_size.as_ptr(),
            max_sets: count as u32,
        };

        let pool = unsafe {
            device.raw().create_descriptor_pool(&pool_info,
                                                None).unwrap()
        };

        let layouts: Vec<vk::DescriptorSetLayout> = (0..count).map(|_| {
            descriptor_layout.layout
        }).collect();

        let descriptor_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: pool,
            descriptor_set_count: count as u32,
            p_set_layouts: layouts.as_ptr(),
        };


        let sets = unsafe {
            device.raw().allocate_descriptor_sets(&descriptor_info).unwrap()
        }.iter().map(|&vk_set| {
            DescriptorSet::new(device.clone(), vk_set)
        }).collect();

        DescriptorSetPool {
            device,
            pool,
            sets,
            current: 0
        }
    }

    pub fn next(&mut self) -> &mut DescriptorSet {
        let size = self.sets.len();

        let set = &mut self.sets[self.current];
        self.current = (self.current + 1) % size;

        set
    }

    pub fn current(&mut self) -> &mut DescriptorSet {
        &mut self.sets[self.current - 1]
    }
}

impl Wrap<vk::DescriptorPool> for DescriptorSetPool {
    fn raw(&self) -> vk::DescriptorPool {
        self.pool
    }
}

impl Drop for DescriptorSetPool {
    fn drop(&mut self) {
        trace!("Drop descriptor pool");
        unsafe {
            self.device.raw().destroy_descriptor_pool(self.pool, None);
        }
    }
}
