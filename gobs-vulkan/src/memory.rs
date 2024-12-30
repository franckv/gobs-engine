use std::mem::align_of;
use std::sync::Arc;

use ash::vk::Handle;
use bytemuck::Pod;
use gpu_allocator::vulkan;

use crate::alloc::Allocator;
use crate::device::Device;

#[allow(unused)]
pub struct Memory {
    pub device: Arc<Device>,
    pub allocator: Arc<Allocator>,
    pub allocation: Option<vulkan::Allocation>,
}

impl Memory {
    pub fn upload<T: Copy>(&mut self, entries: &[T], offset: usize) {
        let size = std::mem::size_of_val(entries) as u64;

        tracing::debug!(
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

    pub fn download<T: Pod>(&self, data: &mut Vec<T>) {
        if let Some(allocation) = &self.allocation {
            if let Some(bytes) = allocation.mapped_slice() {
                data.extend_from_slice(bytemuck::cast_slice(bytes));
            }
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        if let Some(allocation) = &self.allocation {
            unsafe { tracing::debug!("Free memory: {:x}", allocation.memory().as_raw()) };
        }

        self.allocator
            .allocator
            .lock()
            .unwrap()
            .free(self.allocation.take().unwrap())
            .unwrap();
    }
}
