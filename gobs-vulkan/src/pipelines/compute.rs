use std::sync::Arc;

use ash::vk;

use crate::pipelines::{Pipeline, PipelineLayout, Shader, ShaderStage};
use crate::{Wrap, device::Device};

#[derive(Default)]
pub struct ComputePipelineBuilder {
    device: Option<Arc<Device>>,
    pipeline_layout: Option<Arc<PipelineLayout>>,
    compute_stage: Option<ShaderStage>,
}

impl ComputePipelineBuilder {
    pub(crate) fn new(device: Arc<Device>) -> Self {
        ComputePipelineBuilder {
            device: Some(device),
            ..Default::default()
        }
    }

    pub fn compute_shader(mut self, entry: &str, vshader: Shader) -> Self {
        self.compute_stage = Some(ShaderStage::new(entry, vshader));

        self
    }

    pub fn layout(mut self, pipeline_layout: Arc<PipelineLayout>) -> Self {
        self.pipeline_layout = Some(pipeline_layout);

        self
    }

    pub fn build(self) -> Arc<Pipeline> {
        let device = self.device.unwrap();

        let pipeline_layout = self.pipeline_layout.unwrap();
        let compute_stage = self.compute_stage.unwrap();
        let compute_stage_info = compute_stage.info();

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(compute_stage_info)
            .layout(pipeline_layout.raw());
        let pipeline = unsafe {
            device
                .raw()
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .unwrap()[0]
        };

        let bind_point = vk::PipelineBindPoint::COMPUTE;

        Arc::new(Pipeline {
            device,
            layout: pipeline_layout,
            pipeline,
            bind_point,
        })
    }
}
