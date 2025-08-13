use std::sync::Arc;

use gobs_core::logger;
use gobs_vulkan as vk;

use crate::Device;
use crate::GfxError;
use crate::backend::vulkan::{display::VkDisplay, instance::VkInstance, renderer::VkRenderer};
use crate::command::CommandQueueType;

pub struct VkDevice {
    pub(crate) device: Arc<vk::device::Device>,
    pub(crate) graphics_queue: Arc<vk::queue::Queue>,
    pub(crate) transfer_queue: Arc<vk::queue::Queue>,
    pub allocator: Arc<vk::alloc::Allocator>,
}

impl Device<VkRenderer> for VkDevice {
    fn new(instance: Arc<VkInstance>, display: &VkDisplay) -> Result<Arc<Self>, GfxError>
    where
        Self: Sized,
    {
        let expected_features = vk::feature::Features::default()
            .fill_mode_non_solid()
            .shader_draw_parameters()
            .buffer_device_address()
            .descriptor_indexing()
            .dynamic_rendering()
            .synchronization2();

        let p_device = instance
            .instance
            .find_adapter(&expected_features, display.surface.as_deref())
            .ok_or(GfxError::DeviceCreate)?;

        tracing::info!(target: logger::INIT, "Using adapter {}", p_device.name);

        let device = vk::device::Device::new(
            instance.instance.clone(),
            p_device,
            display.surface.as_deref(),
        )?;

        let graphics_queue = device.clone().graphics_queue();
        let transfer_queue = device.clone().transfer_queue();

        let allocator = vk::alloc::Allocator::new(device.clone());

        Ok(Arc::new(Self {
            device,
            graphics_queue,
            transfer_queue,
            allocator,
        }))
    }

    fn wait(&self) {
        self.device.wait();
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
