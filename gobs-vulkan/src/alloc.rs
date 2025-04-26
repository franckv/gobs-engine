use std::sync::{Arc, Mutex};

use ash::vk::{self, Handle};
use gpu_allocator::{AllocatorDebugSettings, MemoryLocation, vulkan};

use crate::{Wrap, buffers::BufferUsage, device::Device, memory::Memory};

pub struct Allocator {
    pub allocator: Mutex<vulkan::Allocator>,
    pub device: Arc<Device>,
}

impl Allocator {
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        let allocator = vulkan::Allocator::new(&vulkan::AllocatorCreateDesc {
            instance: device.instance.cloned(),
            device: device.cloned(),
            physical_device: device.p_device.raw(),
            debug_settings: AllocatorDebugSettings {
                log_memory_information: true,
                log_leaks_on_shutdown: true,
                store_stack_traces: false,
                log_allocations: true,
                log_frees: true,
                log_stack_traces: false,
            },
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })
        .unwrap();

        Arc::new(Allocator {
            allocator: Mutex::new(allocator),
            device,
        })
    }

    pub fn allocate_buffer(
        self: Arc<Self>,
        usage: BufferUsage,
        buffer: vk::Buffer,
        label: &str,
    ) -> Memory {
        let mem_req = unsafe { self.device.raw().get_buffer_memory_requirements(buffer) };
        tracing::debug!("Allocating buffer {}: {:?}", label, mem_req);

        let allocation = self
            .allocator
            .lock()
            .unwrap()
            .allocate(&vulkan::AllocationCreateDesc {
                name: label,
                requirements: mem_req,
                location: usage.into(),
                linear: true,
                allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
            })
            .unwrap();

        unsafe {
            tracing::debug!(
                "Binding memory {:x} with buffer {}",
                allocation.memory().as_raw(),
                label
            );

            self.device
                .raw()
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();
        }

        Memory {
            device: self.device.clone(),
            allocator: self.clone(),
            allocation: Some(allocation),
        }
    }

    pub fn allocate_image(self: Arc<Self>, image: vk::Image, label: &str) -> Memory {
        let mem_req = unsafe { self.device.raw().get_image_memory_requirements(image) };
        tracing::debug!("Allocating image {}: {:?}", label, mem_req);

        let allocation = self
            .allocator
            .lock()
            .unwrap()
            .allocate(&vulkan::AllocationCreateDesc {
                name: label,
                requirements: mem_req,
                location: MemoryLocation::GpuOnly,
                linear: true,
                allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
            })
            .unwrap();

        unsafe {
            tracing::debug!(
                "Binding memory {:x} with image {}",
                allocation.memory().as_raw(),
                label
            );

            self.device
                .raw()
                .bind_image_memory(image, allocation.memory(), allocation.offset())
                .unwrap();
        }

        Memory {
            device: self.device.clone(),
            allocator: self.clone(),
            allocation: Some(allocation),
        }
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop allocator: {:?}", self.allocator.lock().unwrap());
    }
}
