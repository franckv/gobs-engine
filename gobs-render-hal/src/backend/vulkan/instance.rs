use gobs_core::memory::{
    allocator::{Allocation as _, Allocator as _},
    bump::BumpAllocator,
};
use gobs_vulkan::{Buffer, DescriptorStage, DescriptorType};

use crate::{BindResource, BindingGroupLayout, BindingGroupType, Handle};

pub struct InstanceRegistry {
    buffers: Vec<Handle>,
    allocators: Vec<BumpAllocator>,
    buffer_size: usize,
}

impl InstanceRegistry {
    pub fn new(buffers: Vec<Handle>, instance_size: usize, capacity: usize) -> Self {
        let total_size = capacity * instance_size;

        let allocators = buffers
            .iter()
            .map(|_| BumpAllocator::new(total_size))
            .collect();

        Self {
            buffers,
            allocators,
            buffer_size: total_size,
        }
    }

    pub fn reset(&mut self, frame_id: usize) {
        debug_assert!(frame_id < self.allocators.len());

        self.allocators[frame_id].clear();
    }

    pub fn get_buffer(&self, frame_id: usize) -> Handle {
        debug_assert!(frame_id < self.buffers.len());

        self.buffers[frame_id]
    }

    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn allocate(&mut self, frame_id: usize, size: usize) -> usize {
        debug_assert!(frame_id < self.allocators.len());

        self.allocators[frame_id]
            .allocate(size)
            .expect("Not enough instance memory")
            .start()
    }
}
