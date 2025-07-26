use std::sync::Arc;

use ash::vk;

use gobs_core::logger;

use crate::Wrap;
use crate::device::Device;
use crate::queue::QueueFamily;

/// Used to allocate new CommandBuffers
pub struct CommandPool {
    device: Arc<Device>,
    pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: Arc<Device>, queue_family: &QueueFamily) -> Arc<Self> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family.index);

        let pool = unsafe {
            tracing::debug!(target: logger::INIT, "Create command pool");
            device.raw().create_command_pool(&pool_info, None).unwrap()
        };

        Arc::new(CommandPool { device, pool })
    }
}

impl Wrap<vk::CommandPool> for CommandPool {
    fn raw(&self) -> vk::CommandPool {
        self.pool
    }
}
impl Drop for CommandPool {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop command pool");
        unsafe {
            self.device.raw().destroy_command_pool(self.pool, None);
        }
    }
}
