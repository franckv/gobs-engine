use gobs_gfx::{Buffer, BufferId};
use gobs_vulkan as vk;

use crate::{device::VkDevice, renderer::VkRenderer};

#[derive(Debug)]
pub struct VkBuffer {
    id: BufferId,
    pub(crate) buffer: vk::buffer::Buffer,
}

impl Buffer<VkRenderer> for VkBuffer {
    fn id(&self) -> BufferId {
        self.id
    }

    fn new(name: &str, size: usize, usage: vk::buffer::BufferUsage, device: &VkDevice) -> VkBuffer {
        Self {
            id: BufferId::new_v4(),
            buffer: vk::buffer::Buffer::new(
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

    fn address(&self, device: &VkDevice) -> u64 {
        self.buffer.address(device.device.clone())
    }
}
