use std::mem::{self, align_of};
use std::sync::{Arc, Mutex};

use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;

use crate::buffer::BufferUsage;
use crate::device::Device;

#[allow(unused)]
pub struct Memory {
    device: Arc<Device>,
    allocator: Arc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
}

impl Memory {
    pub(crate) fn with_buffer(
        device: Arc<Device>,
        buffer: vk::Buffer,
        usage: BufferUsage,
        allocator: Arc<Mutex<Allocator>>,
    ) -> Self {
        let mem_req = unsafe { device.raw().get_buffer_memory_requirements(buffer) };
        log::debug!("Allocating buffer: {:?}", mem_req);

        let allocation = allocator
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: "buffer",
                requirements: mem_req,
                location: usage.into(),
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .unwrap();

        unsafe {
            device
                .raw()
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();
        }

        Memory {
            device,
            allocator,
            allocation: Some(allocation),
        }
    }

    pub(crate) fn with_image(
        device: Arc<Device>,
        image: vk::Image,
        label: &str,
        allocator: Arc<Mutex<Allocator>>,
    ) -> Self {
        let mem_req = unsafe { device.raw().get_image_memory_requirements(image) };
        log::debug!("Allocating buffer: {:?}", mem_req);

        let allocation = allocator
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: label,
                requirements: mem_req,
                location: MemoryLocation::GpuOnly,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .unwrap();

        unsafe {
            device
                .raw()
                .bind_image_memory(image, allocation.memory(), allocation.offset())
                .unwrap();
        }

        Memory {
            device,
            allocator,
            allocation: Some(allocation),
        }
    }

    pub fn upload<T: Copy>(&mut self, entries: &[T], offset: usize) {
        let size = (entries.len() * mem::size_of::<T>()) as u64;

        log::debug!(
            "Uploading data to buffer (Size={}, offset={}, align={}, len={})",
            size,
            offset,
            align_of::<T>(),
            entries.len()
        );

        if let Some(allocation) = &mut self.allocation {
            presser::copy_from_slice_to_offset_with_align(
                entries,
                allocation,
                offset,
                align_of::<T>(),
            )
            .unwrap();
        } else {
            panic!("No allocation");
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        log::debug!("Free memory");
        self.allocator
            .lock()
            .unwrap()
            .free(self.allocation.take().unwrap())
            .unwrap();
    }
}
