use std::sync::Arc;

use gobs_resource::geometry::VertexAttribute;
use uuid::Uuid;

use gobs_core::ImageFormat;

use crate::{
    BlendMode, CompareOp, CullMode, DynamicStateElem, FrontFace, GfxError, PolygonMode, Rect2D,
    Renderer, Viewport,
};

pub type PipelineId = Uuid;

pub trait Pipeline<R: Renderer> {
    fn name(&self) -> &str;
    fn id(&self) -> PipelineId;
    fn vertex_attributes(&self) -> VertexAttribute;
    fn graphics(name: &str, device: Arc<R::Device>) -> R::GraphicsPipelineBuilder;
    fn compute(name: &str, device: Arc<R::Device>) -> R::ComputePipelineBuilder;
}

pub trait ComputePipelineBuilder<R: Renderer> {
    fn shader(self, filename: &str, entry: &str) -> Result<R::ComputePipelineBuilder, GfxError>;
    fn binding_group(self, binding_group_layout: R::BindingGroupLayout) -> Self;
    fn build(self) -> Arc<R::Pipeline>;
}

pub trait GraphicsPipelineBuilder<R: Renderer> {
    fn vertex_shader(
        self,
        filename: &str,
        entry: &str,
    ) -> Result<R::GraphicsPipelineBuilder, GfxError>;
    fn fragment_shader(
        self,
        filename: &str,
        entry: &str,
    ) -> Result<R::GraphicsPipelineBuilder, GfxError>;
    fn push_constants(self, size: usize) -> Self;
    fn vertex_attributes(self, vertex_attributes: VertexAttribute) -> Self;
    fn binding_group(self, binding_group_layout: R::BindingGroupLayout) -> Self;
    fn polygon_mode(self, mode: PolygonMode) -> Self;
    fn viewports(self, viewports: Vec<Viewport>) -> Self;
    fn scissors(self, scissors: Vec<Rect2D>) -> Self;
    fn dynamic_states(self, states: &[DynamicStateElem]) -> Self;
    fn attachments(
        self,
        color_format: Option<ImageFormat>,
        depth_format: Option<ImageFormat>,
    ) -> Self;
    fn depth_test_disable(self) -> Self;
    fn depth_test_enable(self, write_enable: bool, op: CompareOp) -> Self;
    fn blending_enabled(self, blend_mode: BlendMode) -> Self;
    fn cull_mode(self, cull_mode: CullMode) -> Self;
    fn front_face(self, front_face: FrontFace) -> Self;
    fn build(self) -> Arc<R::Pipeline>;
}
