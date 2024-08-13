use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::{
    BindingGroupType, GfxBindingGroup, GfxComputePipelineBuilder, GfxDevice,
    GfxGraphicsPipelineBuilder,
};

pub type PipelineId = Uuid;

pub trait Pipeline {
    fn name(&self) -> &str;
    fn id(&self) -> PipelineId;
    fn graphics(name: &str, device: &GfxDevice) -> GfxGraphicsPipelineBuilder;
    fn compute(name: &str, device: &GfxDevice) -> GfxComputePipelineBuilder;
    fn create_binding_group(self: &Arc<Self>, ty: BindingGroupType) -> Result<GfxBindingGroup>;
    fn reset_binding_group(self: &Arc<Self>, ty: BindingGroupType);
}
