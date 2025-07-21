#![allow(clippy::new_ret_no_self)]

use std::sync::Arc;

use uuid::Uuid;

use gobs_core::{ImageExtent2D, ImageFormat, Transform};
use gobs_gfx::{GfxPipeline, ImageLayout, ImageUsage};
use gobs_resource::{
    entity::{camera::Camera, light::Light},
    geometry::VertexAttribute,
};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject,
    data::{SceneData, UniformLayout},
    graph::GraphResourceManager,
};

pub mod bounds;
pub mod compute;
pub mod depth;
pub mod dummy;
pub mod forward;
pub mod material;
pub mod present;
pub mod select;
pub mod ui;
pub mod wire;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PassType {
    Bounds,
    Compute,
    Depth,
    Dummy,
    Forward,
    Material,
    Present,
    Select,
    Wire,
    Ui,
}

pub type PassId = Uuid;

#[allow(dead_code)]
#[derive(Default)]
pub enum AttachmentAccess {
    #[default]
    Read,
    Write,
    ReadWrite,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub enum AttachmentType {
    #[default]
    Input,
    Color,
    Depth,
    Resolve,
    Preserve,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Attachment {
    ty: AttachmentType,
    access: AttachmentAccess,
    format: ImageFormat,
    usage: ImageUsage,
    extent: ImageExtent2D,
    layout: ImageLayout,
    clear: bool,
    scaling: f32,
}

impl Attachment {
    pub fn new(ty: AttachmentType, access: AttachmentAccess) -> Self {
        Self {
            ty,
            access,
            scaling: 1.,
            ..Default::default()
        }
    }

    pub fn with_format(&mut self, format: ImageFormat) -> &mut Self {
        self.format = format;

        self
    }

    pub fn with_usage(&mut self, usage: ImageUsage) -> &mut Self {
        self.usage = usage;

        self
    }

    pub fn with_extent(&mut self, extent: ImageExtent2D) -> &mut Self {
        self.extent = extent;

        self
    }

    pub fn with_layout(&mut self, layout: ImageLayout) -> &mut Self {
        self.layout = layout;

        self
    }

    pub fn with_clear(&mut self, clear: bool) -> &mut Self {
        self.clear = clear;

        self
    }

    pub fn scaled_extent(&self) -> ImageExtent2D {
        ImageExtent2D::new(
            (self.extent.width as f32 * self.scaling) as u32,
            (self.extent.height as f32 * self.scaling) as u32,
        )
    }
}

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
        scene_data: &SceneData,
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
