use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;
use uuid::Uuid;

use gobs_core::{logger, utils::timer::Timer};
use gobs_gfx::{Buffer, BufferId, Command, GfxBindingGroup, GfxPipeline, Pipeline, PipelineId};

use crate::{FrameData, GfxContext, ObjectDataLayout, RenderObject, UniformBuffer, UniformLayout};

#[derive(Debug, Error)]
pub enum RenderJobError {
    #[error("invalid pipeline")]
    InvalidPipeline,
}

#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    draws: u32,
    binds: u32,
    cpu_draw_time: f32,
    pub gpu_draw_time: f32,
    timer: Timer,
}

impl RenderStats {
    pub fn reset(&mut self) {
        self.draws = 0;
        self.binds = 0;
        self.timer.reset();
    }

    pub fn draw(&mut self) {
        self.draws += 1;
    }

    pub fn bind(&mut self) {
        self.binds += 1;
    }

    pub fn finish(&mut self) {
        self.cpu_draw_time = self.timer.delta();
    }
}

struct RenderJobState {
    last_pipeline: PipelineId,
    last_index_buffer: BufferId,
    last_indices_offset: usize,
    scene_data_bound: bool,
    object_data: Vec<u8>,
}

impl RenderJobState {
    pub fn new() -> Self {
        Self {
            last_pipeline: PipelineId::nil(),
            last_index_buffer: BufferId::nil(),
            last_indices_offset: 0,
            scene_data_bound: false,
            object_data: vec![],
        }
    }
}

pub struct RenderJob {
    pass_id: Uuid,
    fixed_pipeline: Option<Arc<GfxPipeline>>,
    uniform_buffer: RwLock<UniformBuffer>,
    object_layout: ObjectDataLayout,
    render_transparent: bool,
    render_opaque: bool,
}

impl RenderJob {
    pub fn new(
        ctx: &GfxContext,
        pass_id: Uuid,
        object_layout: ObjectDataLayout,
        scene_data_layout: Arc<UniformLayout>,
        render_transparent: bool,
        render_opaque: bool,
    ) -> Self {
        let uniform_buffer = RwLock::new(UniformBuffer::new(&ctx.device, scene_data_layout));

        Self {
            pass_id,
            fixed_pipeline: None,
            uniform_buffer,
            object_layout,
            render_transparent,
            render_opaque,
        }
    }

    pub fn set_pipeline(&mut self, pipeline: Arc<GfxPipeline>) {
        self.fixed_pipeline = Some(pipeline)
    }

    pub fn update_uniform(&self, uniform_data: &[u8]) {
        self.uniform_buffer.write().update(uniform_data);
    }

    pub fn should_render(&self, render_object: &RenderObject) -> bool {
        tracing::trace!(target: logger::RENDER, "Object render pass {}", render_object.pass_id);
        if render_object.pass_id != self.pass_id
            || (render_object.is_transparent() && !self.render_transparent)
            || (!render_object.is_transparent() && !self.render_opaque)
        {
            tracing::trace!(target: logger::RENDER, "Skip object");
            false
        } else {
            true
        }
    }

    pub fn draw_list(
        &self,
        ctx: &GfxContext,
        frame: &mut FrameData,
        render_list: &[RenderObject],
    ) -> Result<(), RenderJobError> {
        let mut state = RenderJobState::new();

        for render_object in render_list {
            if !self.should_render(render_object) {
                continue;
            }

            let pipeline = self.get_pipeline(render_object)?;

            self.bind_pipeline(frame, &pipeline, &mut state);

            // bind camera and lights
            tracing::debug!(target: logger::RENDER, "Bind scene data");
            if !state.scene_data_bound {
                let uniform_buffer = self.uniform_buffer.read();

                frame
                    .command
                    .bind_resource_buffer(&uniform_buffer.buffer, &pipeline);
                state.scene_data_bound = true;
            }

            // bind materials...
            if self.fixed_pipeline.is_none() {
                tracing::debug!(target: logger::RENDER, "Bind resources");
                for bind_group in &render_object.bind_groups {
                    self.bind_resource(frame, bind_group, &pipeline, &mut state);
                }
            }

            tracing::debug!(target: logger::RENDER, "Bind object data");
            self.bind_object_data(ctx, frame, render_object, &mut state)?;

            tracing::debug!(target: logger::RENDER, "Draw object");
            frame.command.draw_indexed(render_object.indices_len, 1);
            frame.stats.draw();
        }

        frame.stats.finish();

        Ok(())
    }

    fn get_pipeline(
        &self,
        render_object: &RenderObject,
    ) -> Result<Arc<GfxPipeline>, RenderJobError> {
        if let Some(pipeline) = &self.fixed_pipeline {
            tracing::debug!(target: logger::RENDER, "Use fixed pipeline");
            Ok(pipeline.clone())
        } else if let Some(pipeline) = &render_object.pipeline {
            tracing::debug!(target: logger::RENDER, "Use object pipeline");
            Ok(pipeline.clone())
        } else {
            Err(RenderJobError::InvalidPipeline)
        }
    }

    fn bind_resource(
        &self,
        frame: &mut FrameData,
        bind_group: &GfxBindingGroup,
        pipeline: &GfxPipeline,
        _state: &mut RenderJobState,
    ) {
        tracing::trace!(target: logger::RENDER, "Bind resource: {:?} ({:?})", bind_group.bind_group_type, bind_group.ds.layout);
        tracing::trace!(target: logger::RENDER, "Bind pipeline: {:?}", pipeline.pipeline.layout.descriptor_layouts);

        frame.command.bind_resource(bind_group, pipeline);
        frame.stats.bind();
    }

    fn bind_pipeline(
        &self,
        frame: &mut FrameData,
        pipeline: &GfxPipeline,
        state: &mut RenderJobState,
    ) {
        if state.last_pipeline != pipeline.id() {
            tracing::debug!(target: logger::RENDER, "Bind pipeline: {}", pipeline.id());
            frame.command.bind_pipeline(pipeline);
            frame.stats.bind();
            state.last_pipeline = pipeline.id();
        }
    }

    fn bind_object_data(
        &self,
        ctx: &GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        tracing::trace!(target: logger::RENDER, "Bind push constants");

        state.object_data.clear();

        let pipeline = self.get_pipeline(render_object)?;

        tracing::trace!(target: logger::RENDER, "Copy object data: {}", self.object_layout.uniform_layout().size());

        self.object_layout
            .copy_data(ctx, render_object, &mut state.object_data);

        frame.command.push_constants(&pipeline, &state.object_data);

        if state.last_index_buffer != render_object.index_buffer.id()
            || state.last_indices_offset != render_object.indices_offset
        {
            frame
                .command
                .bind_index_buffer(&render_object.index_buffer, render_object.indices_offset);
            frame.stats.bind();
            state.last_index_buffer = render_object.index_buffer.id();
            state.last_indices_offset = render_object.indices_offset;
        }

        Ok(())
    }
}
