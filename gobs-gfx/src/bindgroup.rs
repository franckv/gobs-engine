use serde::Deserialize;
use serde::Serialize;

use crate::DescriptorStage;
use crate::DescriptorType;
use crate::ImageLayout;
use crate::Renderer;

#[derive(Copy, Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub enum BindingGroupType {
    None,
    ComputeData,
    SceneData,
    MaterialData,
    MaterialTextures,
}

impl BindingGroupType {
    #[allow(clippy::match_like_matches_macro)]
    pub fn is_push(&self) -> bool {
        match self {
            BindingGroupType::SceneData => true,
            _ => false,
        }
    }

    pub fn set(&self) -> u32 {
        match self {
            BindingGroupType::None => panic!("Invalid binding group"),
            BindingGroupType::ComputeData => 0,
            BindingGroupType::SceneData => 0,
            BindingGroupType::MaterialData => 1,
            BindingGroupType::MaterialTextures => 2,
        }
    }
}

pub trait BindingGroupLayout<R: Renderer> {
    fn new(binding_group_type: BindingGroupType) -> Self;
    fn add_binding(self, ty: DescriptorType, stage: DescriptorStage) -> Self;
}

pub trait BindingGroupPool<R: Renderer> {
    fn allocate(&mut self) -> R::BindingGroup;
    fn reset(&mut self);
}

pub trait BindingGroup<R: Renderer>: Clone {
    fn update(&self) -> R::BindingGroupUpdates;
}

pub trait BindingGroupUpdates<R: Renderer> {
    fn bind_buffer(self, buffer: &R::Buffer, start: usize, len: usize) -> Self;
    fn bind_image(self, image: &R::Image, layout: ImageLayout) -> Self;
    fn bind_sampled_image(self, image: &R::Image, layout: ImageLayout) -> Self;
    fn bind_sampler(self, sampler: &R::Sampler) -> Self;
    fn end(self);
}
