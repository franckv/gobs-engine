use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use backend::device::Device;
use backend::memory::Memory;
use backend::Wrap;

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
                vk::MemoryPropertyFlags::HOST_VISIBLE |
                    vk::MemoryPropertyFlags::HOST_COHERENT
            },
            BufferUsage::Vertex => {
                vk::MemoryPropertyFlags::DEVICE_LOCAL
            },
            BufferUsage::Instance => {
                vk::MemoryPropertyFlags::HOST_VISIBLE |
                    vk::MemoryPropertyFlags::HOST_COHERENT
            },
            BufferUsage::Index => {
                vk::MemoryPropertyFlags::DEVICE_LOCAL
            },
            BufferUsage::Uniform => {
                vk::MemoryPropertyFlags::HOST_VISIBLE |
                    vk::MemoryPropertyFlags::HOST_COHERENT
            }
        }
    }
}

pub struct Buffer<T> {
    device: Arc<Device>,
    buffer: vk::Buffer,
    memory: Memory,
    count: usize,
    marker: PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new(count: usize, usage: BufferUsage, device: Arc<Device>) -> Self {
        let usage_flags = match usage {
            BufferUsage::Staging => {
                vk::BufferUsageFlags::TRANSFER_SRC
            },
            BufferUsage::Vertex => {
                vk::BufferUsageFlags::TRANSFER_DST |
                    vk::BufferUsageFlags::VERTEX_BUFFER
            },
            BufferUsage::Instance => {
                vk::BufferUsageFlags::VERTEX_BUFFER
            },
            BufferUsage::Index => {
                vk::BufferUsageFlags::TRANSFER_DST |
                    vk::BufferUsageFlags::INDEX_BUFFER
            },
            BufferUsage::Uniform => {
                vk::BufferUsageFlags::UNIFORM_BUFFER
            }
        };

        let size = count * mem::size_of::<T>();

        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            size: size as u64,
            usage: usage_flags,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        let buffer = unsafe {
            device.raw().create_buffer(&buffer_info, None).unwrap()
        };

        let memory = Memory::with_buffer(device.clone(), buffer, usage);

        Buffer {
            device,
            buffer,
            memory,
            count,
            marker: PhantomData,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn size(&self) -> usize {
        self.count * mem::size_of::<T>()
    }

    pub fn item_size(&self) -> usize {
        mem::size_of::<T>()
    }

    pub fn copy(&mut self, entries: &Vec<T>) {
        self.memory.upload(entries);
    }
}

impl<T> Wrap<vk::Buffer> for Buffer<T> {
    fn raw(&self) -> vk::Buffer {
        self.buffer
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        trace!("Drop buffer");
        unsafe {
            self.device.raw().destroy_buffer(self.buffer, None);
        }
    }
}
