pub mod alloc;
pub mod buffers;
pub mod command;
pub(crate) mod debug;
pub mod descriptor;
pub mod device;
pub mod error;
pub mod feature;
pub mod framebuffer;
pub mod images;
pub mod instance;
pub mod memory;
pub mod physical;
pub mod pipelines;
pub mod query;
pub mod queue;
pub mod renderpass;
pub mod surface;
pub mod swapchain;
pub mod sync;

pub use alloc::Allocator;
pub use buffers::{Buffer, BufferUsage};
pub use command::{CommandBuffer, CommandPool};
pub use descriptor::{DescriptorStage, DescriptorType};
pub use device::Device;
pub use feature::Features;
pub use images::{Image, Sampler};
pub use instance::Instance;
pub use pipelines::{
    BlendMode, CompareOp, ComputePipelineBuilder, CullMode, DynamicStateElem, FrontFace,
    GraphicsPipelineBuilder, Pipeline, PolygonMode, Rect2D, Viewport,
};
pub use queue::Queue;

#[cfg(test)]
mod headless;

pub trait Wrap<T> {
    fn raw(&self) -> T;
}
