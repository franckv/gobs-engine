use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::queue::QueueFamily;
use crate::Wrap;

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
            log::debug!("Create command pool");
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
        log::debug!("Drop command pool");
        unsafe {
            self.device.raw().destroy_command_pool(self.pool, None);
        }
    }
}
