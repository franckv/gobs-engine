use std::sync::Arc;

use gobs_core::memory::allocator::Allocator;
use gobs_gfx::{Command, GfxCommand, GfxDevice, GfxPipeline, Pipeline, PipelineId};
use gobs_resource::entity::uniform::UniformLayout;
use thiserror::Error;

use crate::{FrameData, GfxContext, RenderObject, UniformBuffer};

#[derive(Debug, Error)]
pub enum RenderTaskError {
    #[error("invalid pipeline")]
    InvalidPipeline,
}

struct PassFrameData {
    pub uniform_buffer: Allocator<GfxDevice, Arc<UniformLayout>, UniformBuffer>,
}

impl PassFrameData {
    pub fn new() -> Self {
        PassFrameData {
            uniform_buffer: Allocator::new(),
        }
    }
}

pub struct RenderTask {
    pipeline: Option<Arc<GfxPipeline>>,
    last_pipeline: PipelineId,
    frame_data: Vec<PassFrameData>,
    uniform_data_layout: Option<Arc<UniformLayout>>,
}

impl RenderTask {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            last_pipeline: PipelineId::nil(),
            frame_data: Vec::new(),
            uniform_data_layout: None,
        }
    }

    pub fn set_pipeline(&mut self, pipeline: Arc<GfxPipeline>) {
        self.pipeline = Some(pipeline);
    }

    pub fn set_uniform_data_layout(&mut self, layout: Arc<UniformLayout>) {
        self.uniform_data_layout = Some(layout);
    }

    pub fn update_uniform(
        &self,
        ctx: &GfxContext,
        frame: &mut PassFrameData,
        uniform_data: Option<&[u8]>,
    ) {
        if let Some(uniform_data_layout) = &self.uniform_data_layout {
            if let Some(uniform_data) = uniform_data {
                let mut uniform_buffer = frame.uniform_buffer.allocate(
                    &ctx.device,
                    "uniform",
                    1,
                    uniform_data_layout.clone(),
                );
                uniform_buffer.update(uniform_data);
            }
        }
    }

    pub fn bind_pipeline(
        &mut self,
        cmd: &GfxCommand,
        object_pipeline: &Option<Arc<GfxPipeline>>,
    ) -> Result<(), RenderTaskError> {
        if let Some(pipeline) = &self.pipeline {
            if self.last_pipeline != pipeline.id() {
                cmd.bind_pipeline(pipeline);
            }
            self.last_pipeline = pipeline.id();
            Ok(())
        } else if let Some(pipeline) = object_pipeline {
            if self.last_pipeline != pipeline.id() {
                cmd.bind_pipeline(pipeline);
            }
            self.last_pipeline = pipeline.id();
            Ok(())
        } else {
            Err(RenderTaskError::InvalidPipeline)
        }
    }

    pub fn submit(
        &mut self,
        _ctx: &mut GfxContext,
        frame: &FrameData,
        render_list: &[RenderObject],
        _uniform_data: Option<&[u8]>,
    ) -> Result<(), RenderTaskError> {
        let cmd = &frame.command;

        for render_object in render_list {
            self.bind_pipeline(cmd, &render_object.pipeline)?;
        }

        Ok(())
    }
}
