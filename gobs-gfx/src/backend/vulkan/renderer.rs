use crate::{
    Renderer,
    backend::vulkan::{
        bindgroup::{VkBindingGroup, VkBindingGroupUpdates},
        buffer::VkBuffer,
        command::VkCommand,
        device::VkDevice,
        display::VkDisplay,
        image::{VkImage, VkSampler},
        instance::VkInstance,
        pipeline::{VkComputePipelineBuilder, VkGraphicsPipelineBuilder, VkPipeline},
    },
};

pub struct VkRenderer;

impl Renderer for VkRenderer {
    type Device = VkDevice;
    type Display = VkDisplay;
    type Image = VkImage;
    type Instance = VkInstance;
    type BindingGroup = VkBindingGroup;
    type BindingGroupUpdates = VkBindingGroupUpdates;
    type Buffer = VkBuffer;
    type Command = VkCommand;
    type Pipeline = VkPipeline;
    type ComputePipelineBuilder = VkComputePipelineBuilder;
    type GraphicsPipelineBuilder = VkGraphicsPipelineBuilder;
    type Sampler = VkSampler;
}
