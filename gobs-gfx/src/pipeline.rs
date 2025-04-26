use std::sync::Arc;

use uuid::Uuid;

use gobs_core::ImageFormat;

use crate::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType,
    DynamicStateElem, FrontFace, GfxError, PolygonMode, Rect2D, Renderer, Viewport,
};

pub type PipelineId = Uuid;

pub trait Pipeline<R: Renderer> {
    fn name(&self) -> &str;
    fn id(&self) -> PipelineId;
    fn graphics(name: &str, device: &R::Device) -> R::GraphicsPipelineBuilder;
    fn compute(name: &str, device: &R::Device) -> R::ComputePipelineBuilder;
    fn create_binding_group(
        self: &Arc<Self>,
        ty: BindingGroupType,
    ) -> Result<R::BindingGroup, GfxError>;
    fn reset_binding_group(self: &Arc<Self>, ty: BindingGroupType);
}

pub trait ComputePipelineBuilder<R: Renderer> {
    fn shader(self, filename: &str, entry: &str) -> Result<R::ComputePipelineBuilder, GfxError>;
    fn binding_group(self, binding_group_type: BindingGroupType) -> Self;
    fn binding(self, ty: DescriptorType) -> Self;
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
    fn pool_size(self, size: usize) -> Self;
    fn push_constants(self, size: usize) -> Self;
    fn binding_group(self, binding_group_type: BindingGroupType) -> Self;
    fn current_binding_group(&self) -> Option<BindingGroupType>;
    fn binding(self, ty: DescriptorType, stage: DescriptorStage) -> Self;

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
