use slotmap::SlotMap;

use gobs_vulkan as vk;

use crate::{Handle, backend::vulkan::buffer::BufferView};

#[derive(Default)]
pub(crate) struct ResourcesRegistry {
    pub(crate) buffers: SlotMap<Handle, BufferView>,
    pub(crate) images: SlotMap<Handle, vk::Image>,
    pub(crate) samplers: SlotMap<Handle, vk::Sampler>,
    pub(crate) pipelines: SlotMap<Handle, vk::Pipeline>,
}
