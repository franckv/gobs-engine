#![allow(clippy::new_ret_no_self)]

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{ImageLayout, ImageUsage};
use gobs_render_low::{FrameData, GfxContext, RenderError, RenderObject, SceneData, UniformLayout};
use gobs_resource::geometry::VertexAttribute;

use crate::graph::GraphResourceManager;

pub mod compute;
pub mod material;
pub mod present;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum RenderPassType {
    Compute,
    Material,
    Present,
}

pub type PassId = Uuid;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
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
    fn vertex_attributes(&self) -> Option<VertexAttribute>;
    fn push_layout(&self) -> Option<Arc<UniformLayout>>;
    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        render_list: &[RenderObject],
        scene_data: &SceneData,
        draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError>;
}
