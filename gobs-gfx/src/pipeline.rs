use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::BindingGroupType;

pub type PipelineId = Uuid;

pub trait Pipeline {
    type GfxBindingGroup;
    type GfxDevice;
    type GfxComputePipelineBuilder;
    type GfxGraphicsPipelineBuilder;

    fn name(&self) -> &str;
    fn id(&self) -> PipelineId;
    fn graphics(name: &str, device: &Self::GfxDevice) -> Self::GfxGraphicsPipelineBuilder;
    fn compute(name: &str, device: &Self::GfxDevice) -> Self::GfxComputePipelineBuilder;
    fn create_binding_group(
        self: &Arc<Self>,
        ty: BindingGroupType,
    ) -> Result<Self::GfxBindingGroup>;
    fn reset_binding_group(self: &Arc<Self>, ty: BindingGroupType);
}
