use serde::Deserialize;
use serde::Serialize;

use crate::ImageLayout;
use crate::Renderer;

#[derive(Copy, Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub enum BindingGroupType {
    None,
    ComputeData,
    SceneData,
    MaterialTextures,
}

impl BindingGroupType {
    pub fn set(&self) -> u32 {
        match self {
            BindingGroupType::None => panic!("Invalid binding group"),
            BindingGroupType::ComputeData => 0,
            BindingGroupType::SceneData => 0,
            BindingGroupType::MaterialTextures => 2,
        }
    }
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
