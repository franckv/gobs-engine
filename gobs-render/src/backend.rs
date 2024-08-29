pub mod vulkan;

use std::sync::Arc;

use crate::batch;
use crate::context;
use crate::graph;
use crate::material;
use crate::model;
use crate::pass;

#[cfg(feature = "vulkan")]
pub(crate) use vulkan::*;

pub type Context = context::Context<GfxRenderer>;
pub type FrameGraph = graph::FrameGraph<GfxRenderer>;
pub type RenderBatch = batch::RenderBatch<GfxRenderer>;
pub type Material = material::Material<GfxRenderer>;
pub type MaterialInstance = material::MaterialInstance<GfxRenderer>;
pub type RenderPass = Arc<dyn pass::RenderPass<GfxRenderer>>;
pub type Model = model::Model<GfxRenderer>;
