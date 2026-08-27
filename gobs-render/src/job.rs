use std::sync::Arc;

use gobs_core::{
    data::data_buffer::{DataBuffer, FixedBuffer},
    logger,
};
use gobs_render_graph::{FrameData, RenderError};
use gobs_render_hal::{
    AttributeData, BindResource, BindingId, Handle, ObjectDataLayout, ObjectDataProp,
    UniformBuffer, UniformData as _,
};

use crate::{GfxContext, data::RenderFlags, render_object::RenderObject};

const PUSH_SIZE: usize = 128;

struct RenderJobState {
    last_pipeline: Option<Handle>,
    last_index_buffer: Option<Handle>,
    last_material_data: Option<BindingId>,
    last_material_textures: Option<BindingId>,
    last_vertex_buffer: Option<Handle>,
    last_vertex_buffer_address: u64,
    texture_array_bound: bool,
    material_array_bound: bool,
    scene_data_bound: bool,
}

impl RenderJobState {
    pub fn new() -> Self {
        Self {
            last_pipeline: None,
            last_index_buffer: None,
            last_material_data: None,
            last_material_textures: None,
            last_vertex_buffer: None,
            last_vertex_buffer_address: 0,
            texture_array_bound: false,
            material_array_bound: false,
            scene_data_bound: false,
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

#[derive(Clone, Debug)]
pub struct RenderStats {
    pub objects: u32,
    pub draws: u32,
    pub pipelines: u32,
    pub materials: u32,
    pub uniforms: u32,
    pub indexes: u32,
}

impl RenderStats {
    pub fn new() -> Self {
        Self {
            objects: 0,
            draws: 0,
            pipelines: 0,
            materials: 0,
            uniforms: 0,
            indexes: 0,
        }
    }
}

enum InstanceData {
    Buffer(u64),
    Push(FixedBuffer<PUSH_SIZE>),
}

struct DrawCall {
    // render_object.material.pipeline
    pipeline: Handle,

    // render_object.material.material_indexing
    material_indexing: bool,
    // render_object.material.texture_indexing
    texture_indexing: bool,
    // render_object.material.material_data (ds)
    material_data: Option<BindResource>,
    // render_object.material.material_textures (ds)
    material_textures: Option<BindResource>,

    // Instance: render_object.transform
    // Instance: render_object.vertex_buffer
    // Instance: render_object.material.material_offset
    instance_data: InstanceData,
    instance_len: usize,

    // render_object.index_buffer
    index_buffer: Handle,
    // render_object.index_len
    index_len: usize,
}

pub struct RenderJob<'a> {
    state: RenderJobState,
    stats: RenderStats,
    fixed_pipeline: Option<Handle>,
    scene_buffer: Option<&'a UniformBuffer>,
}

impl<'a> RenderJob<'a> {
    pub fn new() -> Self {
        Self {
            state: RenderJobState::new(),
            stats: RenderStats::new(),
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
    ) -> Result<RenderStats, RenderError> {
        let draws = &self.prepare_draws(ctx, frame, pass_name, render_list, render_flags)?;

        for draw in draws {
            self.draw_objects(ctx, frame, draw)?;
        }

        Ok(self.stats.clone())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn prepare_draws(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pass_name: &str,
        render_list: &[RenderObject],
        render_flags: RenderFlags,
    ) -> Result<Vec<DrawCall>, RenderError> {
        let mut draws: Vec<DrawCall> = Vec::with_capacity(render_list.len());
        let instance_handle = ctx.get_instance_buffer(frame.id);

        for render_object in render_list {
            if !Self::should_render(pass_name, render_flags, render_object) {
                tracing::trace!(target: logger::RENDER, "Skip object");
                continue;
            }

            self.stats.objects += 1;

            let pipeline = self.get_pipeline(render_object)?;

            let material_indexing = render_object.material.material_indexing;
            let texture_indexing = render_object.material.texture_indexing;

            let index_buffer = render_object.index_buffer;
            let index_len = render_object.index_len;

            let object_layout = ctx.get_pipeline_object_layout(pipeline);

            let add_to_draw = if let Some(last_instance) = draws.last() {
                object_layout.instancing
                    && last_instance.pipeline == pipeline
                    && last_instance.index_buffer == index_buffer
                    && last_instance.index_len == index_len
                    && ((material_indexing && texture_indexing)
                        || (last_instance.material_data == render_object.material.material_data
                            && last_instance.material_textures
                                == render_object.material.material_textures))
            } else {
                false
            };

            if add_to_draw {
                if let Some(draw) = draws.last_mut()
                    && let InstanceData::Buffer(_) = draw.instance_data
                {
                    draw.instance_len += 1;

                    self.write_instance_data(
                        ctx,
                        frame,
                        render_object,
                        object_layout.clone(),
                        instance_handle,
                    );
                } else {
                    return Err(RenderError::InvalidData);
                }
            } else {
                let instance_data = self.create_instance_data(
                    ctx,
                    frame,
                    render_object,
                    object_layout,
                    instance_handle,
                );

                let draw = DrawCall {
                    pipeline,
                    material_indexing,
                    texture_indexing,
                    material_data: render_object.material.material_data.clone(),
                    material_textures: render_object.material.material_textures.clone(),
                    instance_data,
                    instance_len: 1,
                    index_buffer,
                    index_len,
                };

                draws.push(draw);
            }
        }

        Ok(draws)
    }

    fn copy_object_data<B>(
        ctx: &mut GfxContext,
        render_object: &RenderObject,
        state: &mut RenderJobState,
        object_layout: Arc<ObjectDataLayout>,
        object_data: &mut B,
    ) where
        B: DataBuffer,
    {
        tracing::trace!(target: logger::RENDER, "Copy object data: {} (layout: {:?})", object_layout.uniform_layout().size(), object_layout);

        object_layout.copy_data(object_data, |prop| match prop {
            ObjectDataProp::WorldMatrix => {
                AttributeData::Mat4F(render_object.transform.matrix().to_cols_array_2d())
            }
            ObjectDataProp::VertexBufferAddress => {
                if state.last_vertex_buffer != Some(render_object.vertex_buffer) {
                    state.last_vertex_buffer_address =
                        ctx.get_buffer_address(render_object.vertex_buffer);
                    state.last_vertex_buffer = Some(render_object.vertex_buffer);
                }
                AttributeData::U64(state.last_vertex_buffer_address)
            }
            ObjectDataProp::MaterialOffset => AttributeData::U32(
                render_object
                    .material
                    .material_offset
                    .expect("Material offset is None"),
            ),
            _ => unimplemented!(),
        });
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn create_instance_data(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        object_layout: Arc<ObjectDataLayout>,
        instance_handle: Handle,
    ) -> InstanceData {
        if object_layout.instancing {
            let local_offset =
                self.write_instance_data(ctx, frame, render_object, object_layout, instance_handle);

            let instance_buffer_address = ctx.get_buffer_address(instance_handle);
            let offset = instance_buffer_address + local_offset as u64;

            InstanceData::Buffer(offset)
        } else {
            let mut data = FixedBuffer::<PUSH_SIZE>::new();
            Self::copy_object_data(
                ctx,
                render_object,
                &mut self.state,
                object_layout,
                &mut data,
            );

            InstanceData::Push(data)
        }
    }

    fn write_instance_data(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        render_object: &RenderObject,
        object_layout: Arc<ObjectDataLayout>,
        instance_handle: Handle,
    ) -> usize {
        let instance_size = object_layout.uniform_layout().size();
        let local_offset = ctx.allocate_instance(frame.id, instance_size);

        let mut data = FixedBuffer::<PUSH_SIZE>::new();

        Self::copy_object_data(
            ctx,
            render_object,
            &mut self.state,
            object_layout.clone(),
            &mut data,
        );

        ctx.upload_buffer_with(
            instance_handle,
            local_offset,
            instance_size,
            &mut |_ctx, instance_buffer| {
                instance_buffer.write(data.as_slice());
            },
        );

        local_offset
    }

    fn draw_objects(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        draw: &DrawCall,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Issue draw call");

        self.bind_pipeline(ctx, frame, draw.pipeline)?;

        // bind camera and lights (push, set=0)
        self.bind_scene_data(ctx, frame, draw.pipeline)?;

        if draw.texture_indexing {
            self.bind_texture_array(ctx, frame, draw.pipeline)?;
        }
        if draw.material_indexing {
            self.bind_material_array(ctx, frame, draw.pipeline)?;
        }

        // bind materials (ds, set 1=material, 2=textures)
        self.bind_material_data(
            ctx,
            frame,
            &draw.material_data,
            &draw.material_textures,
            draw.pipeline,
        )?;

        // push constants + index buffer
        self.bind_object_data(
            ctx,
            frame,
            &draw.instance_data,
            draw.index_buffer,
            draw.pipeline,
        )?;

        tracing::trace!(target: logger::RENDER, "Draw object ({})", draw.index_len);
        frame
            .command
            .draw_indexed(draw.index_len, draw.instance_len);
        self.stats.draws += 1;

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
            self.stats.pipelines += 1;
        } else {
            tracing::trace!(target: logger::RENDER, "Skip bind pipeline {:?}={:?}", self.state.last_pipeline, pipeline);
        }

        Ok(())
    }

    fn bind_texture_array(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if self.fixed_pipeline.is_none() && !self.state.texture_array_bound {
            tracing::trace!(target: logger::RENDER, "Bind texture array");
            frame.command.bind_texture_array(ctx, pipeline);

            self.state.texture_array_bound = true;
            self.stats.materials += 1;
        }

        Ok(())
    }

    fn bind_material_array(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if self.fixed_pipeline.is_none() && !self.state.material_array_bound {
            tracing::trace!(target: logger::RENDER, "Bind material array");
            frame.command.bind_material_array(ctx, pipeline);

            self.state.material_array_bound = true;
            self.stats.materials += 1;
        }

        Ok(())
    }

    fn bind_material_data(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        material_data: &Option<BindResource>,
        material_textures: &Option<BindResource>,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        if self.fixed_pipeline.is_none() {
            let material_data_id = material_data.as_ref().map(|bind| bind.id);
            let texture_data_id = material_textures.as_ref().map(|bind| bind.id);

            if let Some(material_data) = material_data
                && self.state.last_material_data != material_data_id
            {
                tracing::trace!(target: logger::RENDER, "Bind material data resources");

                frame.command.bind_resource(ctx, pipeline, material_data);

                self.state.last_material_data = material_data_id;
                self.stats.materials += 1;
            }

            if let Some(material_textures) = material_textures
                && self.state.last_material_textures != texture_data_id
            {
                tracing::trace!(target: logger::RENDER, "Bind material texture resources");

                frame
                    .command
                    .bind_resource(ctx, pipeline, material_textures);

                self.state.last_material_textures = texture_data_id;
                self.stats.materials += 1;
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
                self.stats.uniforms += 1;
            }
            self.state.scene_data_bound = true;
        }

        Ok(())
    }

    fn bind_object_data(
        &mut self,
        ctx: &GfxContext,
        frame: &mut FrameData,
        instance_data: &InstanceData,
        index_buffer: Handle,
        pipeline: Handle,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: logger::RENDER, "Bind push constants");

        match instance_data {
            InstanceData::Buffer(address) => {
                let push_layout = ctx.get_pipeline_push_layout(pipeline);

                let mut data = FixedBuffer::<PUSH_SIZE>::new();

                push_layout.copy_data(&mut data, |prop| match prop {
                    ObjectDataProp::InstanceBufferAddress => AttributeData::U64(*address),
                    _ => unimplemented!(),
                });

                frame.command.push_constants(ctx, pipeline, data.as_slice());
            }
            InstanceData::Push(data) => {
                frame.command.push_constants(ctx, pipeline, data.as_slice());
            }
        }

        if self.state.last_index_buffer != Some(index_buffer) {
            frame.command.bind_index_buffer(ctx, index_buffer);
            self.state.last_index_buffer = Some(index_buffer);
            self.stats.indexes += 1;
        }

        Ok(())
    }
}
