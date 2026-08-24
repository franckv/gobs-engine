use gobs_core::memory::{
    allocator::{Allocation as _, Allocator as _},
    index_pool::IndexPool,
    slab::{SlabAllocation, SlabAllocator},
};
use gobs_vulkan::{DescriptorStage, DescriptorType};

use crate::{BindResource, BindingGroupLayout, BindingGroupType, BufferType, Handle, RenderHAL};

pub struct MaterialRegistry {
    materials: Vec<Option<SlabAllocation>>,
    index_pool: IndexPool,
    binding: BindResource,
    allocator: SlabAllocator,
    material_size: usize,
}

impl MaterialRegistry {
    pub fn new(buffer: Handle, material_size: usize, capacity: usize) -> Self {
        let materials = (0..capacity).map(|_| None).collect();
        let index_pool = IndexPool::new(capacity);

        let layout = BindingGroupLayout::new(BindingGroupType::BindlessMaterial).add_binding(
            DescriptorType::StorageBuffer,
            DescriptorStage::Fragment,
            1,
        );

        let binding = BindResource::new(layout).binding(buffer, 0);

        let allocator = SlabAllocator::new(material_size, capacity);

        Self {
            materials,
            index_pool,
            binding,
            allocator,
            material_size,
        }
    }

    pub fn size(&self) -> usize {
        self.materials.len()
    }

    pub fn material_size(&self, _index: usize) -> usize {
        self.material_size
    }

    pub fn reserve_index(&mut self) -> usize {
        self.index_pool
            .allocate()
            .expect("Not enough texture slots")
    }

    pub fn free(&mut self, index: usize) {
        if let Some(alloc) = self.materials[index].take() {
            self.index_pool.release(index);
            self.allocator.release(alloc);
        }
    }

    pub fn get_offset(&self, index: usize) -> Option<u32> {
        self.materials[index]
            .as_ref()
            .map(|alloc| alloc.start() as u32)
    }

    pub fn get_buffer(&self) -> Handle {
        self.binding
            .slot(0)
            .expect("Material registry buffer not initialized")
    }
}
