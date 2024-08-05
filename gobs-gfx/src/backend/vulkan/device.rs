use std::sync::Arc;

use anyhow::Result;

use gobs_vulkan as vk;

use crate::{
    backend::vulkan::{GfxCommand, VkCommand, VkDisplay, VkInstance},
    Device,
};

pub struct VkDevice {
    pub(crate) device: Arc<vk::device::Device>,
    pub(crate) queue: Arc<vk::queue::Queue>,
    immediate_cmd: VkCommand,
    pub(crate) allocator: Arc<vk::alloc::Allocator>,
}

impl Device for VkDevice {
    fn new(instance: Arc<VkInstance>, display: Arc<VkDisplay>) -> Result<Arc<Self>>
    where
        Self: Sized,
    {
        let (physical_device, queue_family) = match &display.surface {
            Some(surface) => {
                let physical_device = instance.instance.find_adapter(&surface);
                let queue_family = instance
                    .instance
                    .find_family(&physical_device, &surface)
                    .expect("Cannot find queue family");

                (physical_device, queue_family)
            }
            None => {
                let physical_device = instance.instance.find_headless_adapter();
                let queue_family = instance
                    .instance
                    .find_headless_family(&physical_device)
                    .expect("Cannot find queue family");

                (physical_device, queue_family)
            }
        };

        log::info!("Using adapter {}", physical_device.name);

        let device =
            vk::device::Device::new(instance.instance.clone(), physical_device, &queue_family);

        let queue = vk::queue::Queue::new(device.clone(), queue_family);

        let immediate_cmd_pool = vk::command::CommandPool::new(device.clone(), &queue.family);
        let immediate_cmd = VkCommand {
            command: vk::command::CommandBuffer::new(
                device.clone(),
                queue.clone(),
                immediate_cmd_pool,
                "Immediate",
            ),
        };

        let allocator = vk::alloc::Allocator::new(device.clone());

        Ok(Arc::new(Self {
            device,
            queue,
            immediate_cmd,
            allocator,
        }))
    }

    fn run_immediate<F>(&self, callback: F)
    where
        F: Fn(&GfxCommand),
    {
        log::debug!("Submit immediate command");
        let cmd = &self.immediate_cmd.command;

        cmd.fence.reset();
        assert!(!cmd.fence.signaled());

        cmd.reset();

        cmd.begin();

        callback(&self.immediate_cmd);

        cmd.end();

        cmd.submit2(None, None);

        cmd.fence.wait();
        log::debug!("Immediate command done");
    }

    fn run_immediate_mut<F>(&self, mut callback: F)
    where
        F: FnMut(&GfxCommand),
    {
        log::debug!("Submit immediate command");
        let cmd = &self.immediate_cmd.command;

        cmd.fence.reset();
        assert!(!cmd.fence.signaled());

        cmd.reset();

        cmd.begin();

        callback(&self.immediate_cmd);

        cmd.end();

        cmd.submit2(None, None);

        cmd.fence.wait();
        log::debug!("Immediate command done");
    }
}
