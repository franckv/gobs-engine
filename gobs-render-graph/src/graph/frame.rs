use crate::{
    FrameData, GfxContext, GraphConfig, PassId, RenderError, RenderObject, RenderPass,
    data::SceneData,
    graph::resource::GraphResourceManager,
    pass::{Attachment, PassType},
};
use gobs_core::logger;
use gobs_render_hal::{Handle, ImageLayout};

pub struct FrameGraphPass {
    pub pass: RenderPass,
    pub enabled: bool,
}

pub struct FrameGraph {
    pub render_scaling: f32,
    pub passes: Vec<FrameGraphPass>,
    pub attachments: Vec<Attachment>,
    pub resource_manager: GraphResourceManager,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self {
            render_scaling: 1.,
            passes: Vec::new(),
            attachments: Vec::new(),
            resource_manager: GraphResourceManager::new(),
        }
    }

    pub fn load<F>(
        ctx: &mut GfxContext,
        graph_filename: &str,
        graph_name: &str,
        pipeline_resolver: F,
    ) -> Result<Self, RenderError>
    where
        F: FnMut(&str, &mut GfxContext) -> Option<Handle>,
    {
        tracing::debug!(target: logger::INIT, "Load graph: {}", graph_name);
        GraphConfig::load_graph(ctx, graph_filename, graph_name, pipeline_resolver)
            .map_err(|_| RenderError::InvalidData)
    }

    pub fn register_pass(&mut self, pass: RenderPass) {
        let pass = FrameGraphPass {
            pass,
            enabled: true,
        };

        self.passes.push(pass);
    }

    pub fn register_attachment(
        &mut self,
        ctx: &mut GfxContext,
        label: &str,
        attachment: Attachment,
    ) {
        self.resource_manager.register_image(
            ctx,
            label,
            attachment.format,
            attachment.usage,
            attachment.extent,
        );
    }

    pub fn get_pass<F>(&self, cmp: F) -> Result<RenderPass, RenderError>
    where
        F: Fn(&RenderPass) -> bool,
    {
        for pass in &self.passes {
            if cmp(&pass.pass) {
                return Ok(pass.pass.clone());
            }
        }

        Err(RenderError::PassNotFound)
    }

    /*
    pub fn get_image_data<T: Pod>(
        &self,
        ctx: &GfxContext,
        label: &str,
        data: &mut Vec<T>,
        format: ImageFormat,
    ) -> ImageExtent2D {
        ctx.hal.wait();

        let mut src_image = self.resource_manager.image(label);
        let mut mid_image =
            ctx.hal
                .create_image("mid", format, ImageUsage::Color, src_image.extent());
        let mut dst_image =
            ctx.hal
                .create_image("dst", format, ImageUsage::File, src_image.extent());

        let mut buffer = ctx
            .hal
            .create_buffer("copy", dst_image.size(), BufferType::StagingDst);

        let cmd = ctx
            .hal
            .create_command_buffer("Copy command", CommandQueueType::Graphics);

        cmd.run_immediate_mut(label, &|cmd| {
            cmd.transition_image_layout(src_image, ImageLayout::TransferSrc);
            cmd.transition_image_layout(mid_image, ImageLayout::TransferDst);
            let dst_extent = mid_image.extent();
            cmd.copy_image_to_image(&src_image, src_image.extent(), &mut mid_image, dst_extent);

            cmd.transition_image_layout(mid_image, ImageLayout::TransferSrc);
            cmd.transition_image_layout(dst_image, ImageLayout::TransferDst);
            let dst_extent = dst_image.extent();
            cmd.copy_image_to_image(&mid_image, mid_image.extent(), &mut dst_image, dst_extent);

            cmd.transition_image_layout(dst_image, ImageLayout::TransferSrc);
            cmd.copy_image_to_buffer(&dst_image, &mut buffer);
        });

        buffer.get_bytes(data);

        dst_image.extent()
    }
    */

    pub fn pass_by_id(&self, pass_id: PassId) -> Result<RenderPass, RenderError> {
        self.get_pass(|pass| pass.id() == pass_id)
    }

    pub fn pass_by_type(&self, pass_type: PassType) -> Result<RenderPass, RenderError> {
        self.get_pass(|pass| pass.ty() == pass_type)
    }

    pub fn pass_by_name(&self, pass_name: &str) -> Result<RenderPass, RenderError> {
        self.get_pass(|pass| pass.name() == pass_name)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn begin(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
    ) -> Result<(), RenderError> {
        let cmd = &mut frame.command;

        // FIXME: use attachments from graph
        let draw_image_extent = ctx
            .hal()
            .get_image_extent(self.resource_manager.image("draw"));
        if self.resource_manager.resources.contains_key("depth") {
            debug_assert_eq!(
                draw_image_extent,
                ctx.hal()
                    .get_image_extent(self.resource_manager.image("depth"))
            );
        }

        if ctx.hal_mut().acquire(frame.id).is_err() {
            return Err(RenderError::Outdated);
        }

        cmd.reset();

        self.resource_manager.invalidate(ctx.hal_mut());

        cmd.begin(frame.frame_number);

        cmd.begin_label(&format!("Frame {}", frame.frame_number));

        //TODO: cmd.reset_query_pool(&frame.query_pool, 0, 2);
        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::TopOfPipe, 0);

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn end(&mut self, ctx: &mut GfxContext, frame: &mut FrameData) -> Result<(), RenderError> {
        let frame_id = ctx.frame_id(frame.frame_number);
        let cmd = &mut frame.command;

        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::BottomOfPipe, 1);

        if let Some(render_target) = ctx.hal().get_render_target() {
            cmd.transition_image_layout(ctx.hal_mut(), render_target, ImageLayout::Present);
        } else {
            tracing::debug!(target: logger::RENDER, "No render target to present");
        }

        cmd.end_label();

        cmd.end();

        cmd.submit2(ctx.hal(), frame_id);

        let Ok(_) = ctx.hal_mut().present() else {
            tracing::debug!(target: logger::SYNC, "Exit frame: outdated");
            return Err(RenderError::Outdated);
        };

        tracing::debug!(target: logger::SYNC, "End frame");

        Ok(())
    }

    pub fn update(&mut self, _ctx: &GfxContext, _delta: f32) {}

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn render(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        render_list: &[RenderObject],
        scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        for pass in &mut self.passes {
            if !pass.enabled {
                continue;
            }

            let pass = &pass.pass;

            tracing::debug!(target: logger::SYNC, "Begin render pass {}", pass.name());

            let span =
                tracing::span!(target: logger::PROFILE, tracing::Level::TRACE, "Pass", "{}", pass.name())
                    .entered();

            tracing::debug!(target: logger::RENDER, ">>> Begin rendering pass {}", pass.name());

            pass.render(ctx, frame, &self.resource_manager, render_list, scene_data)?;

            tracing::debug!(target: logger::RENDER, "<<< End rendering pass {}", pass.name());
            span.exit();

            tracing::debug!(target: logger::SYNC, "End render pass {}", pass.name());
        }

        Ok(())
    }

    pub fn resize(&mut self, ctx: &mut GfxContext) {
        self.resize_swapchain(ctx);
    }

    fn resize_swapchain(&mut self, ctx: &mut GfxContext) {
        ctx.hal_mut().wait();

        ctx.hal_mut().resize();
    }

    pub fn enable_pass(&mut self, pass_type: PassType, enabled: bool) {
        for pass in &mut self.passes {
            if pass.pass.ty() == pass_type {
                pass.enabled = enabled;
            }
        }
    }
}

impl Default for FrameGraph {
    fn default() -> Self {
        Self::new()
    }
}
