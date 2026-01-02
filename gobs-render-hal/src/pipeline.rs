use gobs_core::ImageFormat;

use crate::{
    BlendMode, CompareOp, CullMode, DynamicStateElem, FrontFace, Handle, PolygonMode, Rect2D,
    RenderHAL, VertexAttribute, Viewport, bindings::BindingGroupLayout,
};

pub trait ComputePipelineBuilder {
    fn shader(self: Box<Self>, filename: &str, entry: &str) -> Box<dyn ComputePipelineBuilder>;
    fn binding_group(
        self: Box<Self>,
        binding_group_layout: BindingGroupLayout,
    ) -> Box<dyn ComputePipelineBuilder>;
    fn build(self: Box<Self>, hal: &mut dyn RenderHAL) -> Handle;
}

pub trait GraphicsPipelineBuilder {
    fn vertex_shader(
        self: Box<Self>,
        filename: &str,
        entry: &str,
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn fragment_shader(
        self: Box<Self>,
        filename: &str,
        entry: &str,
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn push_constants(self: Box<Self>, size: usize) -> Box<dyn GraphicsPipelineBuilder>;
    fn vertex_attributes(
        self: Box<Self>,
        vertex_attributes: VertexAttribute,
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn binding_group(
        self: Box<Self>,
        binding_group_layout: BindingGroupLayout,
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn polygon_mode(self: Box<Self>, mode: PolygonMode) -> Box<dyn GraphicsPipelineBuilder>;
    fn viewports(self: Box<Self>, viewports: Vec<Viewport>) -> Box<dyn GraphicsPipelineBuilder>;
    fn scissors(self: Box<Self>, scissors: Vec<Rect2D>) -> Box<dyn GraphicsPipelineBuilder>;
    fn dynamic_states(
        self: Box<Self>,
        states: &[DynamicStateElem],
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn attachments(
        self: Box<Self>,
        color_format: Option<ImageFormat>,
        depth_format: Option<ImageFormat>,
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn depth_test_disable(self: Box<Self>) -> Box<dyn GraphicsPipelineBuilder>;
    fn depth_test_enable(
        self: Box<Self>,
        write_enable: bool,
        op: CompareOp,
    ) -> Box<dyn GraphicsPipelineBuilder>;
    fn blending_enabled(self: Box<Self>, blend_mode: BlendMode)
    -> Box<dyn GraphicsPipelineBuilder>;
    fn cull_mode(self: Box<Self>, cull_mode: CullMode) -> Box<dyn GraphicsPipelineBuilder>;
    fn front_face(self: Box<Self>, front_face: FrontFace) -> Box<dyn GraphicsPipelineBuilder>;
    fn build(self: Box<Self>, hal: &mut dyn RenderHAL) -> Handle;
}
