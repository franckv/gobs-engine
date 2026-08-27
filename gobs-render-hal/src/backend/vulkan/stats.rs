use std::sync::Arc;

use gobs_vulkan::{
    Device,
    query::{QueryPool, QueryType},
};

pub struct GpuStats {
    pub(crate) query_pool: QueryPool,
}

impl GpuStats {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            query_pool: QueryPool::new(device, QueryType::Timestamp, 2),
        }
    }
}
