use crate::{
    FrameData, GraphConfig, PassMetaData, RenderError, RenderPassType,
    graph::resource::GraphResourceManager, pass::Attachment,
};
use gobs_core::logger;
use gobs_render_hal::{CommandBuffer, GfxContext, ImageLayout, RenderHAL};

pub struct FrameGraphPass {
    pub pass: PassMetaData,
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
        pass_config: F,
    ) -> Result<Self, RenderError>
    where
        F: FnMut(&mut GfxContext, &PassMetaData, RenderPassType),
    {
        tracing::debug!(target: logger::INIT, "Load graph: {}", graph_name);
        GraphConfig::load_graph(ctx, graph_filename, graph_name, pass_config)
            .map_err(|_| RenderError::InvalidData)
    }

    pub fn register_pass(&mut self, pass: PassMetaData, enabled: bool) {
        let pass = FrameGraphPass { pass, enabled };

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

    fn transition_attachments(
        hal: &mut dyn RenderHAL,
        cmd: &mut dyn CommandBuffer,
        resource_manager: &GraphResourceManager,
        pass: &PassMetaData,
    ) {
        for (name, attachment) in &pass.attachments {
            cmd.transition_image_layout(hal, resource_manager.image(name), attachment.layout);
        }
    }

    pub fn get_pass<F>(&self, cmp: F) -> Result<&PassMetaData, RenderError>
    where
        F: Fn(&PassMetaData) -> bool,
    {
        for pass in &self.passes {
            if cmp(&pass.pass) {
                return Ok(&pass.pass);
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

    pub fn pass_by_name(&self, pass_name: &str) -> Result<&PassMetaData, RenderError> {
        self.get_pass(|pass| pass.name == pass_name)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn begin(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
    ) -> Result<(), RenderError> {
        let cmd = &mut frame.command;

        // FIXME: use attachments from graph
        let draw_image_extent = ctx.get_image_extent(self.resource_manager.image("draw"));
        if self.resource_manager.resources.contains_key("depth") {
            debug_assert_eq!(
                draw_image_extent,
                ctx.get_image_extent(self.resource_manager.image("depth"))
            );
        }

        if ctx.acquire(frame.id).is_err() {
            return Err(RenderError::Outdated);
        }

        cmd.reset();

        self.resource_manager.invalidate(ctx);

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

        if let Some(render_target) = ctx.get_render_target() {
            cmd.transition_image_layout(ctx, render_target, ImageLayout::Present);
        } else {
            tracing::debug!(target: logger::RENDER, "No render target to present");
        }

        cmd.end_label();

        cmd.end();

        cmd.submit_graphics(ctx, frame_id);

        let Ok(_) = ctx.present() else {
            tracing::debug!(target: logger::SYNC, "Exit frame: outdated");
            return Err(RenderError::Outdated);
        };

        tracing::debug!(target: logger::SYNC, "End frame");

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn run<F>(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        mut run_pass: F,
    ) -> Result<(), RenderError>
    where
        F: FnMut(
            &mut GfxContext,
            &mut FrameData,
            &GraphResourceManager,
            &PassMetaData,
        ) -> Result<(), RenderError>,
    {
        for pass in &mut self.passes {
            if !pass.enabled {
                tracing::debug!(target: logger::RENDER,
                    "Skip pass: {}", &pass.pass.name);
                continue;
            }

            let pass = &pass.pass;

            Self::transition_attachments(ctx, frame.command.as_mut(), &self.resource_manager, pass);

            tracing::debug!(target: logger::SYNC, "Begin render pass {}", &pass.name);

            let span =
                tracing::span!(target: logger::PROFILE, tracing::Level::TRACE, "Pass", "{}", &pass.name)
                    .entered();

            tracing::debug!(target: logger::RENDER, ">>> Begin rendering pass {}", &pass.name);

            run_pass(ctx, frame, &self.resource_manager, pass)?;

            tracing::debug!(target: logger::RENDER, "<<< End rendering pass {}", &pass.name);
            span.exit();

            tracing::debug!(target: logger::SYNC, "End render pass {}", &pass.name);
        }

        Ok(())
    }

    pub fn enable_pass(&mut self, name: &str, enabled: bool) {
        for pass in &mut self.passes {
            if pass.pass.name == name {
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
