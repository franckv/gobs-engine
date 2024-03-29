use std::mem::{self, align_of};
use std::sync::Arc;

use ash::vk::Handle;
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
        if let Some(allocation) = &self.allocation {
            unsafe { log::debug!("Free memory: {:x}", allocation.memory().as_raw()) };
        }

        self.allocator
            .allocator
            .lock()
            .unwrap()
            .free(self.allocation.take().unwrap())
            .unwrap();
    }
}
