use bytemuck::Pod;

use gobs_vulkan as vk;

use crate::backend::vulkan::{device::VkDevice, renderer::VkRenderer};
use crate::{Buffer, BufferId};

#[derive(Debug)]
pub struct VkBuffer {
    id: BufferId,
    pub(crate) buffer: vk::buffers::Buffer,
}

impl Buffer<VkRenderer> for VkBuffer {
    fn id(&self) -> BufferId {
        self.id
    }

    fn new(
        name: &str,
        size: usize,
        usage: vk::buffers::BufferUsage,
        device: &VkDevice,
    ) -> VkBuffer {
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
