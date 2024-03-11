use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::entity::uniform::UniformLayout;
use gobs_vulkan::{descriptor::DescriptorSet, image::ImageExtent2D, pipeline::Pipeline};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::{RenderError, ResourceManager},
    renderable::RenderBatch,
    resources::UniformBuffer,
    CommandBuffer,
};

pub mod bounds;
pub mod compute;
pub mod depth;
pub mod forward;
pub mod ui;
pub mod wire;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PassType {
    Bounds,
    Compute,
    Depth,
    Forward,
    Wire,
    Ui,
}

pub type PassId = Uuid;

pub trait RenderPass {
    fn id(&self) -> PassId;
    fn name(&self) -> &str;
    fn ty(&self) -> PassType;
    fn pipeline(&self) -> Option<Arc<Pipeline>>;
    fn vertex_flags(&self) -> Option<VertexFlag>;
    fn push_layout(&self) -> Option<Arc<UniformLayout>>;
    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>>;
    fn attachments(&self) -> &[String];
    fn render(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError>;
}

pub(crate) struct FrameData {
    pub uniform_ds: DescriptorSet,
    pub uniform_buffer: RwLock<UniformBuffer>,
}

impl FrameData {
    pub fn new(
        ctx: &Context,
        uniform_layout: Arc<UniformLayout>,
        uniform_ds: DescriptorSet,
    ) -> Self {
        let uniform_buffer = UniformBuffer::new(
            ctx,
            uniform_ds.layout.clone(),
            uniform_layout.size(),
            ctx.allocator.clone(),
        );

        uniform_ds
            .update()
            .bind_buffer(&uniform_buffer.buffer, 0, uniform_buffer.buffer.size)
            .end();

        FrameData {
            uniform_ds,
            uniform_buffer: RwLock::new(uniform_buffer),
        }
    }
}
