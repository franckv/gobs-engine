use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;
use uuid::Uuid;

use gobs_core::logger;
use gobs_gfx::{BindingGroupType, Buffer, BufferId, Command, GfxPipeline, Pipeline, PipelineId};

use crate::{
    FrameData, GfxContext, ObjectDataLayout, RenderObject, UniformBuffer, UniformData,
    UniformLayout, render_object::MaterialInstanceId,
};

#[derive(Debug, Error)]
pub enum RenderJobError {
    #[error("invalid pipeline")]
    InvalidPipeline,
}

struct RenderJobState {
    last_pipeline: PipelineId,
    last_index_buffer: BufferId,
    last_material: MaterialInstanceId,
    last_indices_offset: usize,
    scene_data_bound: bool,
    object_data: Vec<u8>,
}

impl RenderJobState {
    pub fn new() -> Self {
        Self {
            last_pipeline: PipelineId::nil(),
            last_index_buffer: BufferId::nil(),
            last_material: MaterialInstanceId::nil(),
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
        scene_data_layout: &UniformLayout,
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn draw_list(
        &self,
        ctx: &GfxContext,
        frame: &mut FrameData,
        render_list: &[RenderObject],
    ) -> Result<(), RenderJobError> {
        let mut state = RenderJobState::new();

        for render_object in render_list {
            if !self.should_render(render_object) {
                tracing::trace!(target: logger::RENDER, "Skip object");
                continue;
            }

            self.bind_pipeline(frame, render_object, &mut state)?;

            // bind camera and lights (push, set=0)
            self.bind_scene_data(frame, render_object, &mut state)?;

            // bind materials (ds, set 1=material, 2=textures)
            self.bind_material_data(frame, render_object, &mut state)?;

            // push constants + index buffer
            self.bind_object_data(ctx, frame, render_object, &mut state)?;

            tracing::trace!(target: logger::RENDER, "Draw object");
            frame.command.draw_indexed(render_object.indices_len, 1);
            frame
                .stats
                .draw(self.pass_id, render_object.indices_len as u32);
        }

        frame.stats.finish(self.pass_id);

        Ok(())
    }

    fn get_pipeline(
        &self,
        render_object: &RenderObject,
    ) -> Result<Arc<GfxPipeline>, RenderJobError> {
        if let Some(pipeline) = &self.fixed_pipeline {
            tracing::trace!(target: logger::RENDER, "Use fixed pipeline: {}", pipeline.name());
            Ok(pipeline.clone())
        } else if let Some(pipeline) = &render_object.pipeline {
            tracing::trace!(target: logger::RENDER, "Use object pipeline");
            Ok(pipeline.clone())
        } else {
            Err(RenderJobError::InvalidPipeline)
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_pipeline(
        &self,
        frame: &mut FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        tracing::trace!(target: logger::RENDER, "Bind pipeline");
        let pipeline = self.get_pipeline(render_object)?;

        if state.last_pipeline != pipeline.id() {
            tracing::trace!(target: logger::RENDER, "Bind pipeline: {}", pipeline.id());
            frame.command.bind_pipeline(&pipeline);
            frame.stats.bind_pipeline(self.pass_id);
            state.last_pipeline = pipeline.id();
        } else {
            tracing::trace!(target: logger::RENDER, "Skip bind pipeline {}={}", state.last_pipeline, pipeline.id());
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_material_data(
        &self,
        frame: &mut FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        if self.fixed_pipeline.is_none()
            && state.last_material != render_object.material_instance_id
        {
            let pipeline = self.get_pipeline(render_object)?;

            tracing::trace!(target: logger::RENDER, "Bind material resources: {}", render_object.bind_groups.len());
            for bind_group in &render_object.bind_groups {
                tracing::trace!(target: logger::RENDER, "Bind resource: {:?} ({:?})", bind_group.bind_group_type, bind_group.ds.layout);

                frame.command.bind_resource(bind_group, &pipeline);
                frame.stats.bind_material_resource(self.pass_id);
            }

            state.last_material = render_object.material_instance_id;
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_scene_data(
        &self,
        frame: &mut FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        if !state.scene_data_bound {
            tracing::trace!(target: logger::RENDER, "Bind scene data");
            let uniform_buffer = self.uniform_buffer.read();

            let pipeline = self.get_pipeline(render_object)?;

            // bind scene data (push, set 0)
            frame.command.bind_resource_buffer(
                &uniform_buffer.buffer,
                BindingGroupType::SceneData,
                &pipeline,
            );
            frame.stats.bind_scene_resource(self.pass_id);
            state.scene_data_bound = true;
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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
            .copy_data(Some(ctx), render_object, &mut state.object_data);

        frame.command.push_constants(&pipeline, &state.object_data);

        if state.last_index_buffer != render_object.index_buffer.id()
            || state.last_indices_offset != render_object.indices_offset
        {
            frame
                .command
                .bind_index_buffer(&render_object.index_buffer, render_object.indices_offset);
            frame.stats.bind_index_resource(self.pass_id);
            state.last_index_buffer = render_object.index_buffer.id();
            state.last_indices_offset = render_object.indices_offset;
        }

        Ok(())
    }
}
