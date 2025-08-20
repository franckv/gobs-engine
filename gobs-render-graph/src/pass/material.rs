use std::{collections::HashMap, sync::Arc};

use gobs_core::{ImageExtent2D, logger};
use gobs_gfx::{Command, GfxCommand, GfxPipeline, Pipeline};
use gobs_render_low::{
    FrameData, GfxContext, ObjectDataLayout, RenderError, RenderJob, RenderObject, SceneData,
    SceneDataLayout, UniformLayout,
};
use gobs_resource::geometry::VertexAttribute;

use crate::{
    PassId, PassType,
    graph::GraphResourceManager,
    pass::{Attachment, AttachmentAccess, AttachmentType, RenderPass},
};

pub struct MaterialPass {
    id: PassId,
    name: String,
    ty: PassType,
    attachments: HashMap<String, Attachment>,
    input_attachments: Vec<String>,
    color_attachments: Vec<String>,
    depth_attachments: Vec<String>,
    object_layout: ObjectDataLayout,
    scene_layout: SceneDataLayout,
    render_jobs: Vec<RenderJob>,
    fixed_pipeline: Option<Arc<GfxPipeline>>,
}

impl MaterialPass {
    pub fn new(
        ctx: &GfxContext,
        name: &str,
        ty: PassType,
        object_layout: ObjectDataLayout,
        scene_layout: SceneDataLayout,
        render_transparent: bool,
        render_opaque: bool,
    ) -> Self {
        let id = PassId::new_v4();

        let render_jobs = (0..ctx.frames_in_flight)
            .map(|_| {
                RenderJob::new(
                    ctx,
                    id,
                    object_layout.clone(),
                    scene_layout.uniform_layout(),
                    render_transparent,
                    render_opaque,
                )
            })
            .collect();

        Self {
            id,
            name: name.to_string(),
            ty,
            attachments: Default::default(),
            input_attachments: vec![],
            color_attachments: vec![],
            depth_attachments: vec![],
            object_layout,
            scene_layout,
            render_jobs,
            fixed_pipeline: None,
        }
    }

    pub fn set_fixed_pipeline(&mut self, pipeline: Arc<GfxPipeline>) {
        self.fixed_pipeline = Some(pipeline.clone());
        for job in &mut self.render_jobs {
            job.set_pipeline(pipeline.clone());
        }
    }

    pub fn add_attachment(
        &mut self,
        name: &str,
        ty: AttachmentType,
        access: AttachmentAccess,
    ) -> &mut Attachment {
        let attachment = Attachment::new(ty, access);

        match ty {
            AttachmentType::Input => self.input_attachments.push(name.to_string()),
            AttachmentType::Color => self.color_attachments.push(name.to_string()),
            AttachmentType::Depth => self.depth_attachments.push(name.to_string()),
            AttachmentType::Resolve => todo!(),
            AttachmentType::Preserve => todo!(),
        }

        self.attachments.insert(name.to_string(), attachment);

        self.attachments.get_mut(name).expect("insert attachment")
    }

    fn begin_pass(&self, cmd: &GfxCommand, resource_manager: &GraphResourceManager) {
        tracing::debug!(target: logger::RENDER, "Begin material pass {}", &self.name);

        cmd.begin_label(&format!("Draw {}", &self.name));

        let (color_img, color_clear, color_extent) = match self.color_attachments.first() {
            Some(color) => {
                let color_attach = &self.attachments[color];
                (
                    Some(resource_manager.image_read(color)),
                    color_attach.clear,
                    Some(color_attach.scaled_extent()),
                )
            }
            None => (None, false, None),
        };

        let (depth_img, depth_clear, depth_extent) = match self.depth_attachments.first() {
            Some(depth) => {
                let depth_attach = &self.attachments[depth];
                (
                    Some(resource_manager.image_read(depth)),
                    depth_attach.clear,
                    Some(depth_attach.scaled_extent()),
                )
            }
            None => (None, false, None),
        };

        let extent = color_extent.unwrap_or_else(|| depth_extent.unwrap());

        cmd.begin_rendering(
            color_img.as_deref(),
            extent,
            depth_img.as_deref(),
            color_clear,
            depth_clear,
            [0.; 4],
            1.,
        );

        cmd.set_viewport(extent.width, extent.height);
    }

    fn end_pass(&self, cmd: &GfxCommand) {
        cmd.end_rendering();
        cmd.end_label();
    }

    fn transition_attachments(&self, cmd: &GfxCommand, resource_manager: &GraphResourceManager) {
        for (name, attachment) in &self.attachments {
            cmd.transition_image_layout(&mut resource_manager.image_write(name), attachment.layout);
        }
    }
}

impl RenderPass for MaterialPass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn vertex_attributes(&self) -> Option<VertexAttribute> {
        self.fixed_pipeline.as_ref().map(|p| p.vertex_attributes())
    }

    fn push_layout(&self) -> Option<&UniformLayout> {
        Some(self.object_layout.uniform_layout())
    }

    fn render(
        &self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        render_list: &[RenderObject],
        scene_data: &SceneData,
        _draw_extent: ImageExtent2D,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw {}", &self.name());

        self.transition_attachments(&frame.command, resource_manager);

        self.begin_pass(&frame.command, resource_manager);

        tracing::debug!(target: logger::RENDER, "Start render job");
        let render_job = &self.render_jobs[frame.id];

        tracing::debug!(target: logger::RENDER, "Upload scene data");
        let scene_data_bytes = self.scene_layout.data(scene_data);

        tracing::debug!(target: logger::RENDER, "Update Uniform");
        render_job.update_uniform(&scene_data_bytes);

        tracing::debug!(target: logger::RENDER, "Render object list");
        render_job.draw_list(ctx, frame, render_list)?;

        tracing::debug!(target: logger::RENDER, "Stop render job");

        self.end_pass(&frame.command);

        Ok(())
    }
}
