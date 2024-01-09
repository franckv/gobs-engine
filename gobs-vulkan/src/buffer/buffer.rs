use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::memory::Memory;
use crate::Wrap;

pub enum BufferUsage {
    Staging,
    Vertex,
    Instance,
    Index,
    Uniform,
}

impl Into<vk::MemoryPropertyFlags> for BufferUsage {
    fn into(self) -> vk::MemoryPropertyFlags {
        match self {
            BufferUsage::Staging => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            BufferUsage::Vertex => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            BufferUsage::Instance => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            BufferUsage::Index => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            BufferUsage::Uniform => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
        }
    }
}

pub type BufferAddress = vk::DeviceAddress;

/// Data buffer allocated in memory
pub struct Buffer {
    device: Arc<Device>,
    buffer: vk::Buffer,
    memory: Memory,
    pub size: usize,
}

impl Buffer {
    pub fn new(size: usize, usage: BufferUsage, device: Arc<Device>) -> Self {
        let usage_flags = match usage {
            BufferUsage::Staging => vk::BufferUsageFlags::TRANSFER_SRC,
            BufferUsage::Vertex => {
                vk::BufferUsageFlags::TRANSFER_DST
                    | vk::BufferUsageFlags::VERTEX_BUFFER
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            }
            BufferUsage::Instance => vk::BufferUsageFlags::VERTEX_BUFFER,
            BufferUsage::Index => {
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER
            }
            BufferUsage::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
        };

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.raw().create_buffer(&buffer_info, None).unwrap() };

        let memory = Memory::with_buffer(device.clone(), buffer, usage);

        Buffer {
            device,
            buffer,
            memory,
            size,
        }
    }

    pub fn address(&self, device: Arc<Device>) -> BufferAddress {
        let address_info = vk::BufferDeviceAddressInfo::builder()
            .buffer(self.buffer)
            .build();

        unsafe { device.raw().get_buffer_device_address(&address_info) }
    }

    pub fn copy<T: Copy>(&mut self, entries: &[T], offset: usize) {
        self.memory.upload(entries, offset);
    }
}

impl Wrap<vk::Buffer> for Buffer {
    fn raw(&self) -> vk::Buffer {
        self.buffer
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        log::info!("Drop buffer");
        unsafe {
            self.device.raw().destroy_buffer(self.buffer, None);
        }
    }
}
