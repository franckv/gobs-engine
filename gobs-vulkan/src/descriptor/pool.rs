use std::sync::Arc;

use ash::vk::{self, DescriptorPoolResetFlags};

use crate::Wrap;
use crate::descriptor::{DescriptorSet, DescriptorSetLayout};
use crate::device::Device;

const MAX_POOL_SETS: usize = 1024;

#[derive(Debug)]
pub struct DescriptorSetPool {
    device: Arc<Device>,
    pub descriptor_layout: Arc<DescriptorSetLayout>,
    max_sets: usize,
    current_pool: vk::DescriptorPool,
    available_pools: Vec<vk::DescriptorPool>,
    full_pools: Vec<vk::DescriptorPool>,
}

impl DescriptorSetPool {
    pub fn new(
        device: Arc<Device>,
        descriptor_layout: Arc<DescriptorSetLayout>,
        max_sets: usize,
    ) -> Self {
        let current_pool = Self::allocate_pool(device.clone(), descriptor_layout.clone(), max_sets);

        DescriptorSetPool {
            device,
            descriptor_layout,
            max_sets,
            current_pool,
            available_pools: Vec::new(),
            full_pools: Vec::new(),
        }
    }

    fn allocate_pool(
        device: Arc<Device>,
        descriptor_layout: Arc<DescriptorSetLayout>,
        max_sets: usize,
    ) -> vk::DescriptorPool {
        tracing::debug!("Alloc new pool (size={})", max_sets);

        let pool_size: Vec<vk::DescriptorPoolSize> = descriptor_layout
            .bindings
            .iter()
            .map(|binding| vk::DescriptorPoolSize {
                ty: binding.ty.into(),
                descriptor_count: max_sets as u32,
            })
            .collect();

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_size)
            .max_sets(max_sets as u32);

        unsafe {
            device
                .raw()
                .create_descriptor_pool(&pool_info, None)
                .unwrap()
        }
    }

    fn grow(&mut self) {
        tracing::debug!("Growing descriptor pool");

        self.full_pools.push(self.current_pool);

        self.current_pool = self.available_pools.pop().unwrap_or_else(|| {
            self.max_sets = MAX_POOL_SETS.min((self.max_sets as f32 * 1.5) as usize);
            Self::allocate_pool(
                self.device.clone(),
                self.descriptor_layout.clone(),
                self.max_sets,
            )
        });
    }

    pub fn reset(&mut self) {
        tracing::debug!("Reset all descriptor pool");
        Self::reset_pool(self.device.clone(), self.current_pool);

        for pool in self.full_pools.drain(..) {
            Self::reset_pool(self.device.clone(), pool);
            self.available_pools.push(pool);
        }
    }

    fn reset_pool(device: Arc<Device>, pool: vk::DescriptorPool) {
        tracing::debug!("Reset descriptor pool");
        unsafe {
            device
                .raw()
                .reset_descriptor_pool(pool, DescriptorPoolResetFlags::empty())
                .unwrap();
        }
    }

    fn destroy_pool(device: Arc<Device>, pool: vk::DescriptorPool) {
        tracing::debug!("Destroy descriptor pool");
        unsafe {
            device.raw().destroy_descriptor_pool(pool, None);
        }
    }

    fn allocate_ds(&mut self) -> Result<Vec<vk::DescriptorSet>, vk::Result> {
        let layout = self.descriptor_layout.layout;

        let descriptor_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.current_pool)
            .set_layouts(std::slice::from_ref(&layout));

        unsafe { self.device.raw().allocate_descriptor_sets(&descriptor_info) }
    }

    pub fn allocate(&mut self) -> DescriptorSet {
        tracing::debug!("Allocate descriptor set");
        let results = self.allocate_ds();

        let results = match results {
            Err(vk::Result::ERROR_OUT_OF_POOL_MEMORY | vk::Result::ERROR_FRAGMENTED_POOL) => {
                self.grow();
                self.allocate_ds()
            }
            _ => results,
        };

        let mut ds = results
            .expect("Cannot allocate new descriptor set")
            .iter()
            .map(|&vk_set| {
                DescriptorSet::new(self.device.clone(), vk_set, self.descriptor_layout.clone())
            })
            .collect::<Vec<DescriptorSet>>();

        assert_eq!(ds.len(), 1);
        ds.remove(0)
    }
}

impl Wrap<vk::DescriptorPool> for DescriptorSetPool {
    fn raw(&self) -> vk::DescriptorPool {
        self.current_pool
    }
}

impl Drop for DescriptorSetPool {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop descriptor pool");

        Self::destroy_pool(self.device.clone(), self.current_pool);
        for pool in self.available_pools.drain(..) {
            Self::destroy_pool(self.device.clone(), pool);
        }
        for pool in self.full_pools.drain(..) {
            Self::destroy_pool(self.device.clone(), pool);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::descriptor::{
        DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    };
    use tracing::{Level, level_filters::LevelFilter};
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    #[test]
    fn test_alloc() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();

        let ctx = crate::headless::Context::new("test");

        let layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::ImageSampler, DescriptorStage::Compute)
            .binding(DescriptorType::Uniform, DescriptorStage::Compute)
            .binding(DescriptorType::ImageSampler, DescriptorStage::Compute)
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone(), false);

        // pool size: 4/6/9/13/19
        let mut pool = DescriptorSetPool::new(ctx.device.clone(), layout, 4);
        for _ in 0..40 {
            pool.allocate();
        }
        assert_eq!(pool.max_sets, 19);
        assert_eq!(pool.full_pools.len(), 4);
        assert_eq!(pool.available_pools.len(), 0);

        pool.reset();
        assert_eq!(pool.full_pools.len(), 0);
        assert_eq!(pool.available_pools.len(), 4);

        for _ in 0..40 {
            pool.allocate();
        }
        assert_eq!(pool.max_sets, 19);
        assert_eq!(pool.full_pools.len(), 2);
        assert_eq!(pool.available_pools.len(), 2);
    }
}
