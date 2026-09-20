#[allow(unused)]
mod backend;
mod barrier;
mod bindings;
mod command;
mod config;
mod data;
mod error;
mod hal;
mod pipeline;
mod staging;

pub type GfxContext = dyn RenderHAL + 'static;

pub use gobs_vulkan::{
    descriptor::{DescriptorStage, DescriptorType},
    images::{ImageLayout, ImageUsage},
    pipelines::{
        BlendMode, CompareOp, CullMode, DynamicStateElem, FrontFace, PolygonMode, Rect2D, Viewport,
    },
    sync::{BarrierAccess, BarrierStage},
};

pub use barrier::{Barrier, BarrierSyncScope, BarrierTarget, BarrierType};
pub use bindings::{BindResource, BindingGroupLayout, BindingGroupType, BindingId};
pub use command::{CommandBuffer, CommandQueueType};
pub use config::RenderHalConfig;
pub use data::{ObjectDataLayout, ObjectDataProp, UniformBuffer, UniformData, UniformLayout};
pub use error::RenderBackendError;
pub use hal::{BufferType, Handle, RenderHAL, create_hal};
pub use staging::BufferPool;
