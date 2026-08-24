use std::{collections::HashMap, sync::Arc};

use gobs_core::memory::index_pool::IndexPool;
use gobs_vulkan::{DescriptorStage, DescriptorType};

use crate::{BindResource, BindingGroupLayout, BindingGroupType, Handle};

pub struct TextureRegistry {
    textures: Vec<Option<Handle>>,
    index_pool: IndexPool,
    binding: BindResource,
    allocated: HashMap<Handle, usize>,
}

pub(crate) const SAMPLER_BINDING: usize = 0;
pub(crate) const TEXTURES_BINDING: usize = 1;

impl TextureRegistry {
    pub fn new(size: usize, sampler: Handle) -> Self {
        let textures = (0..size).map(|_| None).collect();

        let index_pool = IndexPool::new(size);

        let layout = BindingGroupLayout::new(BindingGroupType::BindlessTextures)
            .add_binding(DescriptorType::Sampler, DescriptorStage::Fragment, 1)
            .add_binding(
                DescriptorType::SampledImage,
                DescriptorStage::Fragment,
                size as u32,
            );

        let binding = BindResource::new(layout).binding(sampler, 0).next();

        debug_assert_eq!(binding.sets(), 2);
        debug_assert_eq!(SAMPLER_BINDING, 0);
        debug_assert_eq!(TEXTURES_BINDING, 1);

        Self {
            textures,
            index_pool,
            binding,
            allocated: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<Handle> {
        self.textures.get(index).copied().flatten()
    }

    pub fn get_binding(&self) -> &BindResource {
        &self.binding
    }

    pub fn size(&self) -> usize {
        self.textures.len()
    }

    pub fn reserve_index(&mut self) -> usize {
        self.index_pool
            .allocate()
            .expect("Not enough texture slots")
    }

    pub fn register(&mut self, texture: Handle) -> usize {
        if let Some(index) = self.allocated.get(&texture) {
            return *index;
        }

        let index = self.reserve_index();

        self.register_with_index(texture, index);

        index
    }

    pub fn register_with_index(&mut self, texture: Handle, index: usize) -> bool {
        if self.textures[index] == Some(texture) {
            return false;
        }

        debug_assert!(!self.allocated.contains_key(&texture));
        debug_assert!(self.textures[index].is_none());

        self.textures[index] = Some(texture);
        self.binding.add_binding(TEXTURES_BINDING, texture, index);
        self.allocated.insert(texture, index);

        true
    }

    pub fn free(&mut self, index: usize) -> Option<Handle> {
        if let Some(handle) = self.textures[index].take() {
            self.binding.remove_binding(TEXTURES_BINDING, index);
            self.index_pool.release(index);
            self.allocated.remove(&handle);

            Some(handle)
        } else {
            None
        }
    }
}
