use std::sync::Arc;

use ash::vk::{self, DescriptorPoolResetFlags};

use crate::descriptor::{DescriptorSet, DescriptorSetLayout};
use crate::device::Device;
use crate::Wrap;

pub struct DescriptorSetPool {
    device: Arc<Device>,
    pool: vk::DescriptorPool,
}

impl DescriptorSetPool {
    pub fn new(
        device: Arc<Device>,
        descriptor_layout: Arc<DescriptorSetLayout>,
        count: u32,
    ) -> Self {
        let pool_size: Vec<vk::DescriptorPoolSize> = descriptor_layout
            .bindings
            .iter()
            .map(|binding| vk::DescriptorPoolSize {
                ty: binding.descriptor_type,
                descriptor_count: count as u32,
            })
            .collect();

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_size)
            .max_sets(count)
            .build();

        let pool = unsafe {
            device
                .raw()
                .create_descriptor_pool(&pool_info, None)
                .unwrap()
        };

        DescriptorSetPool { device, pool }
    }

    pub fn reset(&self) {
        unsafe {
            self.device
                .raw()
                .reset_descriptor_pool(self.pool, DescriptorPoolResetFlags::empty())
                .unwrap()
        }
    }

    pub fn allocate(&self, layout: Arc<DescriptorSetLayout>) -> DescriptorSet {
        let layouts = vec![layout.layout];

        let descriptor_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts)
            .build();

        let mut ds = unsafe {
            self.device
                .raw()
                .allocate_descriptor_sets(&descriptor_info)
                .unwrap()
        }
        .iter()
        .map(|&vk_set| DescriptorSet::new(self.device.clone(), vk_set, layout.clone()))
        .collect::<Vec<DescriptorSet>>();

        ds.remove(0)
    }
}

impl Wrap<vk::DescriptorPool> for DescriptorSetPool {
    fn raw(&self) -> vk::DescriptorPool {
        self.pool
    }
}

impl Drop for DescriptorSetPool {
    fn drop(&mut self) {
        log::debug!("Drop descriptor pool");

        unsafe {
            self.device.raw().destroy_descriptor_pool(self.pool, None);
        }
    }
}
