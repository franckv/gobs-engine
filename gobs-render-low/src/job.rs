use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;

use gobs_gfx::{Buffer, BufferId, Command, GfxCommand, GfxPipeline, Pipeline, PipelineId};
use uuid::Uuid;

use crate::{GfxContext, ObjectDataLayout, RenderObject, UniformBuffer, UniformLayout};

#[derive(Debug, Error)]
pub enum RenderJobError {
    #[error("invalid pipeline")]
    InvalidPipeline,
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
        tracing::trace!(target: "render", "Object render pass {}", render_object.pass_id);
        if render_object.pass_id != self.pass_id
            || (render_object.is_transparent() && !self.render_transparent)
            || (!render_object.is_transparent() && !self.render_opaque)
        {
            tracing::trace!(target: "render", "Skip object");
            false
        } else {
            true
        }
    }

    pub fn draw_list(
        &self,
        ctx: &GfxContext,
        cmd: &GfxCommand,
        render_list: &[RenderObject],
    ) -> Result<(), RenderJobError> {
        let mut state = RenderJobState::new();

        for render_object in render_list {
            if !self.should_render(render_object) {
                continue;
            }

            let pipeline = self.get_pipeline(render_object)?;

            self.bind_pipeline(cmd, &pipeline, &mut state);

            // bind camera and lights
            tracing::debug!(target: "render", "Bind scene data");
            if !state.scene_data_bound {
                let uniform_buffer = self.uniform_buffer.read();

                cmd.bind_resource_buffer(&uniform_buffer.buffer, &pipeline);
                state.scene_data_bound = true;
            }

            // bind materials...
            if self.fixed_pipeline.is_none() {
                tracing::debug!(target: "render", "Bind resources");
                for bind_group in &render_object.bind_groups {
                    cmd.bind_resource(bind_group);
                }
            }

            tracing::debug!(target: "render", "Bind object data");
            self.bind_object_data(ctx, cmd, render_object, &mut state)?;

            tracing::debug!(target: "render", "Draw object");
            cmd.draw_indexed(render_object.indices_len, 1);
        }

        Ok(())
    }

    fn get_pipeline(
        &self,
        render_object: &RenderObject,
    ) -> Result<Arc<GfxPipeline>, RenderJobError> {
        if let Some(pipeline) = &self.fixed_pipeline {
            tracing::debug!(target: "render", "Use fixed pipeline");
            Ok(pipeline.clone())
        } else if let Some(pipeline) = &render_object.pipeline {
            tracing::debug!(target: "render", "Use object pipeline");
            Ok(pipeline.clone())
        } else {
            Err(RenderJobError::InvalidPipeline)
        }
    }

    fn bind_pipeline(&self, cmd: &GfxCommand, pipeline: &GfxPipeline, state: &mut RenderJobState) {
        if state.last_pipeline != pipeline.id() {
            tracing::debug!(target: "render", "Bind pipeline: {}", pipeline.id());
            cmd.bind_pipeline(pipeline);
            state.last_pipeline = pipeline.id();
        }
    }

    fn bind_object_data(
        &self,
        ctx: &GfxContext,
        cmd: &GfxCommand,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        tracing::trace!(target: "render", "Bind push constants");

        state.object_data.clear();

        let pipeline = self.get_pipeline(render_object)?;

        tracing::trace!(target: "render", "Copy object data: {}", self.object_layout.uniform_layout().size());

        self.object_layout
            .copy_data(ctx, render_object, &mut state.object_data);

        cmd.push_constants(&pipeline, &state.object_data);

        if state.last_index_buffer != render_object.index_buffer.id()
            || state.last_indices_offset != render_object.indices_offset
        {
            cmd.bind_index_buffer(&render_object.index_buffer, render_object.indices_offset);
            state.last_index_buffer = render_object.index_buffer.id();
            state.last_indices_offset = render_object.indices_offset;
        }

        Ok(())
    }
}
