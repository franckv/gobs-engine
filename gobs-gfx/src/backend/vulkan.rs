pub(crate) mod bindgroup;
pub(crate) mod buffer;
pub(crate) mod command;
pub(crate) mod device;
pub(crate) mod display;
pub(crate) mod image;
pub(crate) mod instance;
pub(crate) mod pipeline;

use bindgroup::VkBindingGroup;
use buffer::VkBuffer;
use command::VkCommand;
use device::VkDevice;
use display::VkDisplay;
use image::{VkImage, VkSampler};
use instance::VkInstance;
use pipeline::{VkComputePipelineBuilder, VkGraphicsPipelineBuilder, VkPipeline};

pub type GfxCommand = VkCommand;
pub type GfxBuffer = VkBuffer;
pub type GfxDevice = VkDevice;
pub type GfxDisplay = VkDisplay;
pub type GfxImage = VkImage;
pub type GfxInstance = VkInstance;
pub type GfxBindingGroup = VkBindingGroup;
pub type GfxPipeline = VkPipeline;
pub type GfxComputePipelineBuilder = VkComputePipelineBuilder;
pub type GfxGraphicsPipelineBuilder = VkGraphicsPipelineBuilder;
pub type GfxSampler = VkSampler;
