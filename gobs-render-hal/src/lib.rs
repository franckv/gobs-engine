#[allow(unused)]
mod backend;
mod bindings;
mod command;
mod data;
mod error;
mod hal;
mod pipeline;

pub use gobs_vulkan::{
    descriptor::{DescriptorStage, DescriptorType},
    images::{ImageLayout, ImageUsage},
    pipelines::{
        BlendMode, CompareOp, CullMode, DynamicStateElem, FrontFace, PolygonMode, Rect2D, Viewport,
    },
};

pub use bindings::{BindResource, BindingGroupLayout, BindingGroupType};
pub use command::{CommandBuffer, CommandQueueType};
pub use data::{
    ObjectDataLayout, ObjectDataProp, UniformBuffer, UniformData, UniformLayout, UniformProp,
    UniformPropData, VertexAttribute, VertexData,
};
pub use error::RenderBackendError;
pub use hal::{BufferType, Handle, RenderHAL, create_hal};
