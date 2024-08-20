use std::sync::Arc;

use gobs_gfx::{BufferId, PipelineId};
use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::{utils::timer::Timer, ImageExtent2D, Transform};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformLayout},
    geometry::VertexFlag,
};

use crate::{
    batch::RenderBatch,
    context::Context,
    graph::{RenderError, ResourceManager},
    material::MaterialId,
    resources::UniformBuffer,
    GfxCommand, GfxPipeline,
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
    fn pipeline(&self) -> Option<Arc<GfxPipeline>>;
    fn vertex_flags(&self) -> Option<VertexFlag>;
    fn push_layout(&self) -> Option<Arc<UniformLayout>>;
    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>>;
    fn attachments(&self) -> &[String];
    fn color_clear(&self) -> bool;
    fn depth_clear(&self) -> bool;
    fn render(
        &self,
        ctx: &Context,
        cmd: &GfxCommand,
        resource_manager: &ResourceManager,
        batch: &mut RenderBatch,
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

pub(crate) struct RenderState {
    last_pipeline: PipelineId,
    last_index_buffer: BufferId,
    last_material: MaterialId,
    last_indices_offset: usize,
    scene_data_bound: bool,
    object_data: Vec<u8>,
    timer: Timer,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            last_pipeline: PipelineId::nil(),
            last_index_buffer: BufferId::nil(),
            last_material: MaterialId::nil(),
            last_indices_offset: 0,
            scene_data_bound: false,
            object_data: Vec::new(),
            timer: Timer::new(),
        }
    }
}

pub(crate) struct FrameData {
    pub uniform_buffer: RwLock<UniformBuffer>,
}

impl FrameData {
    pub fn new(ctx: &Context, uniform_layout: Arc<UniformLayout>) -> Self {
        let uniform_buffer = UniformBuffer::new(&ctx.device, uniform_layout.size());

        FrameData {
            uniform_buffer: RwLock::new(uniform_buffer),
        }
    }
}
