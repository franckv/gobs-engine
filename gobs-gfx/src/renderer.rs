use crate::{
    Buffer, Command, Device, Display, Image, Instance, Pipeline, Sampler,
    bindgroup::{BindingGroup, BindingGroupUpdates},
    pipeline::{ComputePipelineBuilder, GraphicsPipelineBuilder},
};

pub trait Renderer: Sized {
    type BindingGroup: BindingGroup<Self>;
    type BindingGroupUpdates: BindingGroupUpdates<Self>;
    type Buffer: Buffer<Self>;
    type Command: Command<Self>;
    type Device: Device<Self>;
    type Display: Display<Self>;
    type Image: Image<Self>;
    type Instance: Instance<Self>;
    type Pipeline: Pipeline<Self>;
    type GraphicsPipelineBuilder: GraphicsPipelineBuilder<Self>;
    type ComputePipelineBuilder: ComputePipelineBuilder<Self>;
    type Sampler: Sampler<Self>;
}
