use gobs_core::{
    data::fixed_buffer::{DataBuffer as _, FixedBuffer},
    logger,
};
use gobs_render_graph::{FrameData, RenderError};
use gobs_render_hal::{
    AttributeData, BindingId, Handle, ObjectDataProp, UniformBuffer, UniformData as _,
};

use crate::{GfxContext, data::RenderFlags, render_object::RenderObject};

struct RenderJobState {
    last_pipeline: Option<Handle>,
    last_index_buffer: Option<Handle>,
    last_material_data: Option<BindingId>,
    last_material_textures: Option<BindingId>,
    texture_array_bound: bool,
    material_array_bound: bool,
    scene_data_bound: bool,
    object_data: FixedBuffer<128>,
}

impl RenderJobState {
    pub fn new() -> Self {
        Self {
            last_pipeline: None,
            last_index_buffer: None,
            last_material_data: None,
            last_material_textures: None,
            texture_array_bound: false,
            material_array_bound: false,
            scene_data_bound: false,
            object_data: FixedBuffer::new(),
        }
    }

    pub fn switch_pipeline(&mut self, pipeline: Handle) {
        self.last_pipeline = Some(pipeline);
        self.scene_data_bound = false;
        self.texture_array_bound = false;
        self.last_material_data = None;
        self.last_material_textures = None;
    }
}

pub struct RenderJob<'a> {
    state: RenderJobState,
    fixed_pipeline: Option<Handle>,
    scene_buffer: Option<&'a UniformBuffer>,
}

impl<'a> RenderJob<'a> {
    pub fn new() -> Self {
        let state = RenderJobState::new();

        Self {
            state,
            fixed_pipeline: None,
            scene_buffer: None,
        }
    }

    pub fn with_pipeline(mut self, fixed_pipeline: Option<Handle>) -> Self {
        self.fixed_pipeline = fixed_pipeline;

        self
    }

    pub fn with_scene_buffer(mut self, scene_buffer: &'a UniformBuffer) -> Self {
        self.scene_buffer = Some(scene_buffer);

        self
    }

    pub fn should_render(
        pass_name: &str,
        pass_render_flags: RenderFlags,
        render_object: &RenderObject,
    ) -> bool {
        if !render_object.render_flags.contains(pass_render_flags) {
            tracing::trace!(target: logger::RENDER, "[{}] Skip object {}, object flags: {:?}, pass flags: {:?}", pass_name, &render_object.model, render_object.render_flags, pass_render_flags);
            false
        } else {
            tracing::trace!(target: logger::RENDER, "[{}] Draw object {}, object flags: {:?}, pass flags: {:?}", pass_name, &render_object.model, render_object.render_flags, pass_render_flags);
            true
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn draw_list(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pass_name: &str,
        render_list: &[RenderObject],
        render_flags: RenderFlags,
    ) -> Result<(), RenderError> {
        for render_object in render_list {
            if !Self::should_render(pass_name, render_flags, render_object) {
                tracing::trace!(target: logger::RENDER, "Skip object");
                continue;
            }

            tracing::debug!(target: logger::RENDER, "Render model:  {}", &render_object.model);

            let pipeline = self.get_pipeline(render_object)?;

            self.bind_pipeline(ctx, frame, pipeline)?;

            // bind camera and lights (push, set=0)
            self.bind_scene_data(ctx, frame, pipeline)?;

            self.bind_texture_array(ctx, frame, render_object, pipeline)?;
            self.bind_material_array(ctx, frame, render_object, pipeline)?;

            // bind materials (ds, set 1=material, 2=textures)
            self.bind_material_data(ctx, frame, render_object, pipeline)?;

            // push constants + index buffer
            self.bind_object_data(ctx, frame, render_object, pipeline)?;

            tracing::trace!(target: logger::RENDER, "Draw object ({})", render_object.index_len);
            frame.command.draw_indexed(render_object.index_len, 1);
        }

        Ok(())
    }

    fn get_pipeline(&self, render_object: &RenderObject) -> Result<Handle, RenderError> {
        if let Some(pipeline) = self.fixed_pipeline {
            tracing::trace!(target: logger::RENDER, "Use fixed pipeline");
            Ok(pipeline)
        } else if let Some(pipeline) = render_object.material.pipeline {
            tracing::trace!(target: logger::RENDER, "Use object pipeline");
            Ok(pipeline)
        } else {
            Err(RenderError::InvalidData)
        }
    }

    fn bind_pipeline(
        &mut self,
        ctx: &GfxContext,
        frame: &mut FrameData,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: logger::RENDER, "Bind pipeline");

        if self.state.last_pipeline != Some(pipeline) {
            tracing::trace!(target: logger::RENDER, "Bind pipeline: {:?}", pipeline);
            frame.command.bind_pipeline(ctx, pipeline);
            self.state.switch_pipeline(pipeline);
        } else {
            tracing::trace!(target: logger::RENDER, "Skip bind pipeline {:?}={:?}", self.state.last_pipeline, pipeline);
        }

        Ok(())
    }

    fn bind_texture_array(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if self.fixed_pipeline.is_none()
            && render_object.material.texture_indexing
            && !self.state.texture_array_bound
        {
            tracing::trace!(target: logger::RENDER, "Bind texture array");
            frame.command.bind_texture_array(ctx, pipeline);

            self.state.texture_array_bound = true;
        }

        Ok(())
    }

    fn bind_material_array(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if self.fixed_pipeline.is_none()
            && render_object.material.material_indexing
            && !self.state.material_array_bound
        {
            tracing::trace!(target: logger::RENDER, "Bind material array");
            frame.command.bind_material_array(ctx, pipeline);

            self.state.material_array_bound = true;
        }

        Ok(())
    }

    fn bind_material_data(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if self.fixed_pipeline.is_none() {
            let material_data_id = render_object
                .material
                .material_data
                .as_ref()
                .map(|bind| bind.id);
            let texture_data_id = render_object
                .material
                .material_textures
                .as_ref()
                .map(|bind| bind.id);

            if let Some(material_data) = &render_object.material.material_data
                && self.state.last_material_data != material_data_id
            {
                tracing::trace!(target: logger::RENDER, "Bind material data resources");

                frame.command.bind_resource(ctx, pipeline, material_data);

                self.state.last_material_data = material_data_id
            }

            if let Some(material_textures) = &render_object.material.material_textures
                && self.state.last_material_textures != texture_data_id
            {
                tracing::trace!(target: logger::RENDER, "Bind material texture resources");

                frame
                    .command
                    .bind_resource(ctx, pipeline, material_textures);

                self.state.last_material_textures = texture_data_id;
            }
        }

        Ok(())
    }

    fn bind_scene_data(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if !self.state.scene_data_bound {
            tracing::trace!(target: logger::RENDER, "Bind scene data");

            // bind scene data (push, set 0)
            if let Some(uniform_buffer) = &self.scene_buffer {
                frame
                    .command
                    .bind_resource(ctx, pipeline, &uniform_buffer.buffer);
            }
            self.state.scene_data_bound = true;
        }

        Ok(())
    }

    fn bind_object_data(
        &mut self,
        ctx: &GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: logger::RENDER, "Bind push constants");

        self.state.object_data.clear();

        let object_layout = ctx.get_pipeline_object_layout(pipeline);

        tracing::trace!(target: logger::RENDER, "Copy object data: {} (layout: {:?})", object_layout.uniform_layout().size(), object_layout);

        object_layout.copy_data(&mut self.state.object_data, |prop| match prop {
            ObjectDataProp::WorldMatrix => {
                AttributeData::Mat4F(render_object.transform.matrix().to_cols_array_2d())
            }
            ObjectDataProp::VertexBufferAddress => {
                let vertex_buffer_address = ctx.get_buffer_address(render_object.vertex_buffer);
                AttributeData::U64(vertex_buffer_address)
            }
            ObjectDataProp::MaterialOffset => AttributeData::U32(
                render_object
                    .material
                    .material_offset
                    .expect("Material offset is None"),
            ),
        });

        // TODO: check pipeline object layout compatibility
        frame
            .command
            .push_constants(ctx, pipeline, self.state.object_data.as_slice());

        if self.state.last_index_buffer != Some(render_object.index_buffer) {
            frame
                .command
                .bind_index_buffer(ctx, render_object.index_buffer);
            self.state.last_index_buffer = Some(render_object.index_buffer);
        }

        Ok(())
    }
}
