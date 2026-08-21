use gobs_vulkan::{DescriptorStage, DescriptorType};

use crate::{BindResource, BindingGroupLayout, BindingGroupType, Handle};

pub struct TextureRegistry {
    textures: Vec<Option<Handle>>,
    free_list: Vec<usize>,
    binding: BindResource,
}

const SAMPLER_BINDING: usize = 0;
const TEXTURES_BINDING: usize = 1;

impl TextureRegistry {
    pub fn new(size: usize, sampler: Handle) -> Self {
        let free_list = (0..size).rev().collect();
        let textures = (0..size).map(|_| None).collect();

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
            free_list,
            binding,
        }
    }

    pub fn get(&self, index: usize) -> Option<Handle> {
        self.textures.get(index).copied().flatten()
    }

    pub fn register(&mut self, texture: Handle) -> usize {
        assert!(!self.free_list.is_empty(), "Not enough texture slots");

        let index = self.free_list.pop().unwrap();

        self.textures[index] = Some(texture);

        self.binding.add_binding(TEXTURES_BINDING, texture, index);

        index
    }

    pub fn free(&mut self, index: usize) -> Option<Handle> {
        if let Some(handle) = self.textures[index].take() {
            self.binding.remove_binding(TEXTURES_BINDING, index);

            self.free_list.push(index);

            Some(handle)
        } else {
            None
        }
    }
}
