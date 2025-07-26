use bytemuck::Pod;

use gobs_core::logger;
use gobs_core::memory::allocator::Allocable;
use gobs_vulkan as vk;
use gobs_vulkan::buffers::BufferUsage;

use crate::backend::vulkan::{device::VkDevice, renderer::VkRenderer};
use crate::{Buffer, BufferId};

#[derive(Debug)]
pub struct VkBuffer {
    id: BufferId,
    pub(crate) buffer: vk::buffers::Buffer,
}

impl Buffer<VkRenderer> for VkBuffer {
    fn new(
        name: &str,
        size: usize,
        usage: vk::buffers::BufferUsage,
        device: &VkDevice,
    ) -> VkBuffer {
        tracing::debug!(target: logger::RESOURCES, "Create buffer {}, size={}", name, size);

        Self {
            id: BufferId::new_v4(),
            buffer: vk::buffers::Buffer::new(
                name,
                size,
                usage,
                device.device.clone(),
                device.allocator.clone(),
            ),
        }
    }

    fn id(&self) -> BufferId {
        self.id
    }

    fn copy<T: Copy>(&mut self, entries: &[T], offset: usize) {
        self.buffer.copy(entries, offset);
    }

    fn size(&self) -> usize {
        self.buffer.size
    }

    fn usage(&self) -> vk::buffers::BufferUsage {
        self.buffer.usage
    }

    fn address(&self, device: &VkDevice) -> u64 {
        self.buffer.address(device.device.clone())
    }

    fn get_bytes<T: Pod>(&self, data: &mut Vec<T>) {
        self.buffer.get_bytes(data);
    }
}

impl Allocable<VkDevice, BufferUsage> for VkBuffer {
    fn allocate(device: &VkDevice, name: &str, size: usize, family: BufferUsage) -> Self {
        VkBuffer::new(name, size, family, device)
    }

    fn resource_id(&self) -> BufferId {
        self.id
    }

    fn family(&self) -> BufferUsage {
        self.usage()
    }

    fn resource_size(&self) -> usize {
        self.size()
    }
}
