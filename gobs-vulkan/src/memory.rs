use std::mem;
use std::mem::align_of;
use std::ptr;
use std::sync::Arc;

use ash::util::Align;
use ash::vk;

use log::trace;

use crate::buffer::BufferUsage;
use crate::device::Device;
use crate::Wrap;

pub struct Memory {
    device: Arc<Device>,
    memory: vk::DeviceMemory,
}

impl Memory {
    pub(crate) fn with_buffer(device: Arc<Device>,
                              buffer: vk::Buffer,
                              usage: BufferUsage) -> Self {
        let mem_req = unsafe {
            device.raw().get_buffer_memory_requirements(buffer)
        };

        let memory = Self::allocate(&device, mem_req, usage.into());

        unsafe {
            device.raw().bind_buffer_memory(buffer, memory,
                                            0).unwrap();
        }

        Memory {
            device,
            memory,
        }
    }

    pub(crate) fn with_image(device: Arc<Device>, image: vk::Image) -> Self {
        let mem_req = unsafe {
            device.raw().get_image_memory_requirements(image)
        };

        let mem_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;

        let memory = Self::allocate(&device, mem_req, mem_flags);

        unsafe {
            device.raw().bind_image_memory(image, memory,
                                           0).unwrap();
        }

        Memory {
            device,
            memory,
        }
    }

    fn allocate(device: &Arc<Device>,
                mem_req: vk::MemoryRequirements,
                mem_flags: vk::MemoryPropertyFlags) -> vk::DeviceMemory {
        let mem_type = device.p_device.find_memory_type(&mem_req,
                                                        mem_flags);

        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_req.size,
            memory_type_index: mem_type,
        };

        unsafe {
            device.raw().allocate_memory(&memory_info,
                                         None).unwrap()
        }
    }

    pub fn upload<T: Copy>(&self, entries: &Vec<T>) {
        let size = (entries.len() * mem::size_of::<T>()) as u64;

        let data = unsafe {
            self.device.raw().map_memory(self.memory, 0, size,
                                         vk::MemoryMapFlags::empty()).unwrap()
        };

        let mut align = unsafe {
            Align::new(data, align_of::<T>() as u64, size)
        };

        align.copy_from_slice(entries.as_ref());

        unsafe {
            self.device.raw().unmap_memory(self.memory);
        }
    }
}

impl Wrap<vk::DeviceMemory> for Memory {
    fn raw(&self) -> vk::DeviceMemory {
        self.memory
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        trace!("Free memory");
        unsafe {
            self.device.raw().free_memory(self.memory, None);
        }
    }
}
