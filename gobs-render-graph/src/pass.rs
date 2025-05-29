#![allow(clippy::new_ret_no_self)]

use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{BufferId, GfxPipeline, PipelineId};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformLayout},
    geometry::VertexAttribute,
};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject, UniformBuffer, graph::GraphResourceManager,
};

pub mod bounds;
pub mod compute;
pub mod depth;
pub mod dummy;
pub mod forward;
pub mod present;
pub mod ui;
pub mod wire;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PassType {
    Bounds,
    Compute,
    Depth,
    Dummy,
    Forward,
    Present,
    Wire,
    Ui,
}

pub type PassId = Uuid;

pub trait RenderPass {
    fn id(&self) -> PassId;
    fn name(&self) -> &str;
    fn ty(&self) -> PassType;
    fn pipeline(&self) -> Option<Arc<GfxPipeline>>;
    fn vertex_attributes(&self) -> Option<VertexAttribute>;
    fn push_layout(&self) -> Option<Arc<UniformLayout>>;
    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>>;
    fn attachments(&self) -> &[String];
    fn color_clear(&self) -> bool;
    fn depth_clear(&self) -> bool;
    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        resource_manager: &GraphResourceManager,
        render_list: &[RenderObject],
        uniform_data: Option<&[u8]>,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError>;
    fn get_uniform_data(
        &self,
        camera: &Camera,
        camera_transform: &Transform,
        light: &Light,
        light_transform: &Transform,
    ) -> Vec<u8>;
}

#[derive(Default)]
pub(crate) struct RenderState {
    last_pipeline: PipelineId,
    last_index_buffer: BufferId,
    last_indices_offset: usize,
    scene_data_bound: bool,
    object_data: Vec<u8>,
}

pub(crate) struct PassFrameData {
    pub uniform_buffer: RwLock<UniformBuffer>,
}

impl PassFrameData {
    pub fn new(ctx: &GfxContext, uniform_layout: Arc<UniformLayout>) -> Self {
        let uniform_buffer = UniformBuffer::new(ctx, uniform_layout.size());

        PassFrameData {
            uniform_buffer: RwLock::new(uniform_buffer),
        }
    }
}
