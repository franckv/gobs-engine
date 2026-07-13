use glam::Mat3;
use gobs_core::logger;
use gobs_render_hal::{
    BindResource, BindingGroupLayout, BindingGroupType, DescriptorStage, DescriptorType, Handle,
    ObjectDataProp, UniformBuffer, UniformData as _, UniformLayout, UniformPropData,
};
use thiserror::Error;

use crate::{FrameData, GfxContext, RenderFlags, RenderObject};

#[derive(Debug, Error)]
pub enum RenderJobError {
    #[error("invalid pipeline")]
    InvalidPipeline,
}

struct RenderJobState {
    last_pipeline: Option<Handle>,
    last_index_buffer: Option<Handle>,
    last_material_data: Option<BindResource>,
    last_material_textures: Option<BindResource>,
    scene_data_bound: bool,
    object_data: Vec<u8>,
}

impl RenderJobState {
    pub fn new() -> Self {
        Self {
            last_pipeline: None,
            last_index_buffer: None,
            last_material_data: None,
            last_material_textures: None,
            scene_data_bound: false,
            object_data: vec![],
        }
    }
}

pub struct RenderJob {
    pass_name: String,
    fixed_pipeline: Option<Handle>,
    uniform_buffer: UniformBuffer,
    render_flags: RenderFlags,
}

impl RenderJob {
    pub fn new(
        ctx: &mut GfxContext,
        pass_name: String,
        scene_data_layout: &UniformLayout,
        render_flags: RenderFlags,
    ) -> Self {
        let uniform_bindgroup = BindingGroupLayout::new(BindingGroupType::SceneData)
            .add_binding(DescriptorType::Uniform, DescriptorStage::All);
        let uniform_buffer =
            UniformBuffer::new(ctx.hal.as_mut(), uniform_bindgroup, scene_data_layout);

        Self {
            pass_name,
            fixed_pipeline: None,
            uniform_buffer,
            render_flags,
        }
    }

    pub fn set_pipeline(&mut self, pipeline: Handle) {
        self.fixed_pipeline = Some(pipeline);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn update_uniform(&self, ctx: &mut GfxContext, uniform_data: &[u8]) {
        self.uniform_buffer.update(ctx.hal.as_mut(), uniform_data);
    }

    pub fn should_render(&self, render_object: &RenderObject) -> bool {
        if !render_object.render_flags.intersects(self.render_flags) {
            tracing::trace!(target: logger::RENDER, "[{}] Skip object {}, object flags: {:?}, pass flags: {:?}", &self.pass_name, &render_object.model, render_object.render_flags, self.render_flags);
            false
        } else {
            tracing::trace!(target: logger::RENDER, "[{}] Draw object {}, object flags: {:?}, pass flags: {:?}", &self.pass_name, &render_object.model, render_object.render_flags, self.render_flags);
            true
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn draw_list(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        render_list: &[RenderObject],
    ) -> Result<(), RenderJobError> {
        let mut state = RenderJobState::new();

        for render_object in render_list {
            if !self.should_render(render_object) {
                tracing::trace!(target: logger::RENDER, "Skip object");
                continue;
            }

            tracing::debug!(target: logger::RENDER, "Render model:  {}", &render_object.model);

            self.bind_pipeline(ctx, frame, render_object, &mut state)?;

            // bind camera and lights (push, set=0)
            self.bind_scene_data(ctx, frame, render_object, &mut state)?;

            // bind materials (ds, set 1=material, 2=textures)
            self.bind_material_data(ctx, frame, render_object, &mut state)?;

            // push constants + index buffer
            self.bind_object_data(ctx, frame, render_object, &mut state)?;

            tracing::trace!(target: logger::RENDER, "Draw object ({})", render_object.index_len);
            frame.command.draw_indexed(render_object.index_len, 1);
        }

        Ok(())
    }

    fn get_pipeline(&self, render_object: &RenderObject) -> Result<Handle, RenderJobError> {
        if let Some(pipeline) = self.fixed_pipeline {
            tracing::trace!(target: logger::RENDER, "Use fixed pipeline");
            Ok(pipeline)
        } else if let Some(pipeline) = render_object.pipeline {
            tracing::trace!(target: logger::RENDER, "Use object pipeline");
            Ok(pipeline)
        } else {
            Err(RenderJobError::InvalidPipeline)
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_pipeline(
        &self,
        ctx: &GfxContext,
        frame: &FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        tracing::trace!(target: logger::RENDER, "Bind pipeline");
        let pipeline = self.get_pipeline(render_object)?;

        if state.last_pipeline != Some(pipeline) {
            tracing::trace!(target: logger::RENDER, "Bind pipeline: {:?}", pipeline);
            frame.command.bind_pipeline(ctx.hal.as_ref(), pipeline);
            state.last_pipeline = Some(pipeline);
            state.scene_data_bound = false;
        } else {
            tracing::trace!(target: logger::RENDER, "Skip bind pipeline {:?}={:?}", state.last_pipeline, pipeline);
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_material_data(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        if self.fixed_pipeline.is_none() {
            let pipeline = self.get_pipeline(render_object)?;

            if let Some(material_data) = &render_object.material_data
                && state.last_material_data != render_object.material_data
            {
                tracing::trace!(target: logger::RENDER, "Bind material data resources");

                frame
                    .command
                    .bind_resource(ctx.hal.as_mut(), pipeline, material_data);

                state.last_material_data = render_object.material_data.clone();
            }

            if let Some(material_textures) = &render_object.material_textures
                && state.last_material_textures != render_object.material_textures
            {
                tracing::trace!(target: logger::RENDER, "Bind material texture resources");

                frame
                    .command
                    .bind_resource(ctx.hal.as_mut(), pipeline, material_textures);

                state.last_material_textures = render_object.material_textures.clone();
            }
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_scene_data(
        &self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        if !state.scene_data_bound {
            tracing::trace!(target: logger::RENDER, "Bind scene data");

            let pipeline = self.get_pipeline(render_object)?;

            // bind scene data (push, set 0)
            frame
                .command
                .bind_resource(ctx.hal.as_mut(), pipeline, &self.uniform_buffer.buffer);
            state.scene_data_bound = true;
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn bind_object_data(
        &self,
        ctx: &GfxContext,
        frame: &FrameData,
        render_object: &RenderObject,
        state: &mut RenderJobState,
    ) -> Result<(), RenderJobError> {
        tracing::trace!(target: logger::RENDER, "Bind push constants");

        state.object_data.clear();

        let pipeline = self.get_pipeline(render_object)?;
        let object_layout = ctx.hal.get_pipeline_object_layout(pipeline);

        tracing::trace!(target: logger::RENDER, "Copy object data: {} (layout: {:?})", object_layout.uniform_layout().size(), object_layout);

        object_layout.copy_data(&mut state.object_data, |prop| match prop {
            ObjectDataProp::WorldMatrix => {
                UniformPropData::Mat4F(render_object.transform.matrix().to_cols_array_2d())
            }
            ObjectDataProp::NormalMatrix => UniformPropData::Mat3F(
                Mat3::from_quat(render_object.transform.rotation()).to_cols_array_2d(),
            ),
            ObjectDataProp::VertexBufferAddress => {
                let vertex_buffer_address = ctx.hal.get_buffer_address(render_object.vertex_buffer);
                UniformPropData::U64(vertex_buffer_address)
            }
        });

        // TODO: check pipeline object layout compatibility
        frame
            .command
            .push_constants(ctx.hal.as_ref(), pipeline, &state.object_data);

        if state.last_index_buffer != Some(render_object.index_buffer) {
            frame
                .command
                .bind_index_buffer(ctx.hal.as_ref(), render_object.index_buffer);
            state.last_index_buffer = Some(render_object.index_buffer);
        }

        Ok(())
    }
}
