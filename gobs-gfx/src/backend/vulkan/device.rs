use std::sync::Arc;

use anyhow::Result;

use gobs_vulkan as vk;

use crate::Device;
use crate::backend::vulkan::{
    command::VkCommand, display::VkDisplay, instance::VkInstance, renderer::VkRenderer,
};
use crate::command::CommandQueueType;

pub struct VkDevice {
    pub(crate) device: Arc<vk::device::Device>,
    pub(crate) graphics_queue: Arc<vk::queue::Queue>,
    pub(crate) transfer_queue: Arc<vk::queue::Queue>,
    immediate_cmd: VkCommand,
    transfer_cmd: VkCommand,
    pub allocator: Arc<vk::alloc::Allocator>,
}

impl Device<VkRenderer> for VkDevice {
    fn new(instance: Arc<VkInstance>, display: &VkDisplay) -> Result<Arc<Self>>
    where
        Self: Sized,
    {
        let expected_features = vk::feature::Features::default()
            .fill_mode_non_solid()
            .buffer_device_address()
            .descriptor_indexing()
            .dynamic_rendering()
            .synchronization2();

        let p_device = instance
            .instance
            .find_adapter(&expected_features, display.surface.as_deref())
            .expect("Find suitable adapter");

        let (graphics_family, transfer_family) = instance
            .instance
            .find_family(&p_device, display.surface.as_deref());

        tracing::info!(target: "init", "Using adapter {}", p_device.name);
        tracing::debug!(target: "init", "Using queue families Graphics={:?}, Transfer={:?}", &graphics_family, &transfer_family);

        let device = vk::device::Device::new(
            instance.instance.clone(),
            p_device,
            &graphics_family,
            &transfer_family,
        );

        let transfer_queue_index = if transfer_family.index != graphics_family.index {
            0
        } else {
            1
        };
        let graphics_queue = vk::queue::Queue::new(device.clone(), graphics_family, 0);
        let transfer_queue =
            vk::queue::Queue::new(device.clone(), transfer_family, transfer_queue_index);

        let immediate_cmd_pool =
            vk::command::CommandPool::new(device.clone(), &graphics_queue.family);
        let immediate_cmd = VkCommand {
            command: vk::command::CommandBuffer::new(
                device.clone(),
                graphics_queue.clone(),
                immediate_cmd_pool,
                "Immediate",
            ),
        };

        let transfer_cmd_pool =
            vk::command::CommandPool::new(device.clone(), &transfer_queue.family);
        let transfer_cmd = VkCommand {
            command: vk::command::CommandBuffer::new(
                device.clone(),
                transfer_queue.clone(),
                transfer_cmd_pool,
                "Transfer",
            ),
        };

        let allocator = vk::alloc::Allocator::new(device.clone());

        Ok(Arc::new(Self {
            device,
            graphics_queue,
            transfer_queue,
            immediate_cmd,
            transfer_cmd,
            allocator,
        }))
    }

    #[tracing::instrument(target = "gpu", skip_all, level = "debug")]
    fn run_transfer<F>(&self, callback: F)
    where
        F: Fn(&VkCommand),
    {
        let cmd = &self.transfer_cmd.command;

        cmd.fence.reset();

        cmd.begin();

        callback(&self.transfer_cmd);

        cmd.end();

        cmd.submit2(None, None);

        cmd.fence.wait();
        assert!(cmd.fence.signaled());
    }

    #[tracing::instrument(target = "gpu", skip_all, level = "debug")]
    fn run_transfer_mut<F>(&self, mut callback: F)
    where
        F: FnMut(&VkCommand),
    {
        let cmd = &self.transfer_cmd.command;

        cmd.fence.reset();

        cmd.begin();

        callback(&self.transfer_cmd);

        cmd.end();

        cmd.submit2(None, None);

        cmd.fence.wait();
        assert!(cmd.fence.signaled());
    }

    #[tracing::instrument(target = "gpu", skip_all, level = "debug")]
    fn run_immediate<F>(&self, callback: F)
    where
        F: Fn(&VkCommand),
    {
        let cmd = &self.immediate_cmd.command;

        cmd.fence.reset();

        cmd.begin();

        callback(&self.immediate_cmd);

        cmd.end();

        cmd.submit2(None, None);

        cmd.fence.wait();
        assert!(cmd.fence.signaled());
    }

    fn run_immediate_mut<F>(&self, mut callback: F)
    where
        F: FnMut(&VkCommand),
    {
        tracing::debug!("Submit immediate command");
        let cmd = &self.immediate_cmd.command;

        cmd.fence.reset();
        assert!(!cmd.fence.signaled());

        cmd.reset();

        cmd.begin();

        callback(&self.immediate_cmd);

        cmd.end();

        cmd.submit2(None, None);

        cmd.fence.wait();

        tracing::debug!("Immediate command done");
    }

    fn wait(&self) {
        self.device.wait();
    }

    fn wait_transfer(&self) {
        self.transfer_cmd.command.fence.wait();
    }
}

impl VkDevice {
    pub fn get_queue(&self, ty: CommandQueueType) -> Arc<vk::queue::Queue> {
        match ty {
            CommandQueueType::Graphics => self.graphics_queue.clone(),
            CommandQueueType::Transfer => self.transfer_queue.clone(),
            _ => unimplemented!(),
        }
    }
}
