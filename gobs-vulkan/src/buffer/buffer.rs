use std::fmt::Debug;
use std::sync::Arc;

use ash::vk::{self, Handle};
use bytemuck::Pod;
use gpu_allocator::MemoryLocation;

use crate::alloc::Allocator;
use crate::device::Device;
use crate::memory::Memory;
use crate::{debug, Wrap};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BufferUsage {
    Staging,
    StagingDst,
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
            BufferUsage::StagingDst => {
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

impl Into<MemoryLocation> for BufferUsage {
    fn into(self) -> MemoryLocation {
        match self {
            BufferUsage::Staging => MemoryLocation::CpuToGpu,
            BufferUsage::StagingDst => MemoryLocation::GpuToCpu,
            BufferUsage::Vertex => MemoryLocation::GpuOnly,
            BufferUsage::Instance => MemoryLocation::CpuToGpu,
            BufferUsage::Index => MemoryLocation::GpuOnly,
            BufferUsage::Uniform => MemoryLocation::CpuToGpu,
        }
    }
}

impl Into<vk::BufferUsageFlags> for BufferUsage {
    fn into(self) -> vk::BufferUsageFlags {
        match self {
            BufferUsage::Staging => vk::BufferUsageFlags::TRANSFER_SRC,
            BufferUsage::StagingDst => vk::BufferUsageFlags::TRANSFER_DST,
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
        }
    }
}

pub type BufferAddress = vk::DeviceAddress;

/// Data buffer allocated in memory
pub struct Buffer {
    label: String,
    device: Arc<Device>,
    buffer: vk::Buffer,
    memory: Memory,
    pub size: usize,
    pub usage: BufferUsage,
}

impl Buffer {
    pub fn new(
        label: &str,
        size: usize,
        usage: BufferUsage,
        device: Arc<Device>,
        allocator: Arc<Allocator>,
    ) -> Self {
        let usage_flags = usage.into();

        let buffer_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.raw().create_buffer(&buffer_info, None).unwrap() };

        let buffer_label = format!("[Buffer] {}", label);

        debug::add_label(device.clone(), &buffer_label, buffer);

        let memory = allocator.allocate_buffer(usage, buffer, &buffer_label);

        tracing::debug!("Create buffer {} [{:x}]", buffer_label, buffer.as_raw());

        Buffer {
            label: buffer_label,
            device,
            buffer,
            memory,
            size,
            usage,
        }
    }

    pub fn address(&self, device: Arc<Device>) -> BufferAddress {
        let address_info = vk::BufferDeviceAddressInfo::default().buffer(self.buffer);

        unsafe { device.raw().get_buffer_device_address(&address_info) }
    }

    pub fn copy<T: Copy>(&mut self, entries: &[T], offset: usize) {
        self.memory.upload(entries, offset);
    }

    pub fn get_bytes<T: Pod>(&self, vec: &mut Vec<T>) {
        self.memory.download(vec);
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Buffer {}", self.label)
    }
}

impl Wrap<vk::Buffer> for Buffer {
    fn raw(&self) -> vk::Buffer {
        self.buffer
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop buffer {}", self.label);
        unsafe {
            self.device.raw().destroy_buffer(self.buffer, None);
        }
    }
}
