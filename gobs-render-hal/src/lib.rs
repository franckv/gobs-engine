#[allow(unused)]
mod backend;
mod bindings;
mod command;
mod config;
mod data;
mod error;
mod hal;
mod pipeline;
mod staging;

pub type GfxContext<'a> = dyn RenderHAL + 'a;

pub use gobs_vulkan::{
    descriptor::{DescriptorStage, DescriptorType},
    images::{ImageLayout, ImageUsage},
    pipelines::{
        BlendMode, CompareOp, CullMode, DynamicStateElem, FrontFace, PolygonMode, Rect2D, Viewport,
    },
};

pub use bindings::{BindResource, BindingGroupLayout, BindingGroupType, BindingId};
pub use command::{CommandBuffer, CommandQueueType};
pub use config::RenderHalConfig;
pub use data::{
    AlignMode, Attribute, AttributeData, ObjectDataLayout, ObjectDataProp, UniformBuffer,
    UniformData, UniformLayout, VertexAttribute, VertexData,
};
pub use error::RenderBackendError;
pub use hal::{BufferType, Handle, RenderHAL, create_hal};
pub use staging::BufferPool;
