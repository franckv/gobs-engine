use gobs_vulkan as vk;

use crate::backend::vulkan::device::VkDevice;
use crate::Buffer;

pub struct VkBuffer {
    pub(crate) buffer: vk::buffer::Buffer,
}

impl Buffer for VkBuffer {
    fn new(
        name: &str,
        size: usize,
        usage: gobs_vulkan::buffer::BufferUsage,
        device: &VkDevice,
    ) -> VkBuffer {
        Self {
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
