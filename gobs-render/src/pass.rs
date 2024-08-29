use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::{BufferId, PipelineId, Renderer};
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

pub trait RenderPass<R: Renderer> {
    fn id(&self) -> PassId;
    fn name(&self) -> &str;
    fn ty(&self) -> PassType;
    fn pipeline(&self) -> Option<Arc<R::Pipeline>>;
    fn vertex_flags(&self) -> Option<VertexFlag>;
    fn push_layout(&self) -> Option<Arc<UniformLayout>>;
    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>>;
    fn attachments(&self) -> &[String];
    fn color_clear(&self) -> bool;
    fn depth_clear(&self) -> bool;
    fn render(
        &self,
        ctx: &mut Context<R>,
        cmd: &R::Command,
        resource_manager: &ResourceManager<R>,
        batch: &mut RenderBatch<R>,
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
    last_material: MaterialId,
    last_indices_offset: usize,
    scene_data_bound: bool,
    object_data: Vec<u8>,
}

pub(crate) struct FrameData<R: Renderer> {
    pub uniform_buffer: RwLock<UniformBuffer<R>>,
}

impl<R: Renderer> FrameData<R> {
    pub fn new(ctx: &Context<R>, uniform_layout: Arc<UniformLayout>) -> Self {
        let uniform_buffer = UniformBuffer::new(ctx, uniform_layout.size());

        FrameData {
            uniform_buffer: RwLock::new(uniform_buffer),
        }
    }
}
