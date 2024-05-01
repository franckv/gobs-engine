use std::sync::Arc;

use ash::vk;

use crate::device::Device;

pub enum QueryType {
    Timestamp,
}

impl Into<vk::QueryType> for QueryType {
    fn into(self) -> vk::QueryType {
        match self {
            QueryType::Timestamp => vk::QueryType::TIMESTAMP,
        }
    }
}

pub struct QueryPool {
    pub device: Arc<Device>,
    pub pool: vk::QueryPool,
    pub period: f32,
}

impl QueryPool {
    pub fn new(device: Arc<Device>, ty: QueryType, count: u32) -> Self {
        let create_info = vk::QueryPoolCreateInfo::default()
            .query_type(ty.into())
            .query_count(count);

        let pool = unsafe { device.raw().create_query_pool(&create_info, None).unwrap() };

        let period = device.p_device.props.limits.timestamp_period;

        Self {
            device,
            pool,
            period,
        }
    }

    pub fn get_query_pool_results(&self, first_query: u32, buf: &mut [u64]) {
        unsafe {
            self.device
                .raw()
                .get_query_pool_results(self.pool, first_query, buf, vk::QueryResultFlags::TYPE_64)
                .unwrap();
        }
    }
}

impl Drop for QueryPool {
    fn drop(&mut self) {
        log::debug!("Drop query pool");

        unsafe {
            self.device.raw().destroy_query_pool(self.pool, None);
        }
    }
}
