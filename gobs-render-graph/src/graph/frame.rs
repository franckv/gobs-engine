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
        // TODO: image creation should be deferred to the renderer
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
        tracing::debug!(target: logger::SYNC, "Transition attachment for pass {}", &pass.name);
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

    pub fn pass_by_name(&self, pass_name: &str) -> Result<&PassMetaData, RenderError> {
        self.get_pass(|pass| pass.name == pass_name)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn begin(&mut self, ctx: &mut GfxContext) -> Result<(), RenderError> {
        // FIXME: use attachments from graph
        let draw_image_extent = ctx.get_image_extent(self.resource_manager.image("draw"));
        if self.resource_manager.resources.contains_key("depth") {
            debug_assert_eq!(
                draw_image_extent,
                ctx.get_image_extent(self.resource_manager.image("depth"))
            );
        }

        self.resource_manager.invalidate(ctx);

        Ok(())
    }

    fn run_pass<F>(
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pass: &mut FrameGraphPass,
        resource_manager: &GraphResourceManager,
        mut run_pass_cb: F,
    ) -> Result<(), RenderError>
    where
        F: FnMut(
            &mut GfxContext,
            &mut FrameData,
            &GraphResourceManager,
            &PassMetaData,
        ) -> Result<(), RenderError>,
    {
        if !pass.enabled {
            tracing::debug!(target: logger::RENDER,
                    "Skip pass: {}", &pass.pass.name);
            return Ok(());
        }

        let pass = &pass.pass;

        Self::transition_attachments(ctx, frame.command.as_mut(), resource_manager, pass);

        tracing::debug!(target: logger::SYNC, "Begin render pass {}", &pass.name);

        let span =
                tracing::span!(target: logger::PROFILE, tracing::Level::TRACE, "Pass", "{}", &pass.name)
                    .entered();

        tracing::debug!(target: logger::RENDER, ">>> Begin rendering pass {}", &pass.name);

        run_pass_cb(ctx, frame, resource_manager, pass)?;

        tracing::debug!(target: logger::RENDER, "<<< End rendering pass {}", &pass.name);
        span.exit();

        tracing::debug!(target: logger::SYNC, "End render pass {}", &pass.name);

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn end(&mut self, ctx: &mut GfxContext, frame: &mut FrameData) -> Result<(), RenderError> {
        let cmd = &mut frame.command;

        if let Some(render_target) = ctx.get_render_target() {
            cmd.transition_image_layout(ctx, render_target, ImageLayout::Present);
        } else {
            tracing::debug!(target: logger::RENDER, "No render target to present");
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn run<F>(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        mut run_pass_cb: F,
    ) -> Result<(), RenderError>
    where
        F: FnMut(
            &mut GfxContext,
            &mut FrameData,
            &GraphResourceManager,
            &PassMetaData,
        ) -> Result<(), RenderError>,
    {
        self.begin(ctx)?;

        for pass in &mut self.passes {
            Self::run_pass(ctx, frame, pass, &self.resource_manager, &mut run_pass_cb)?;
        }

        self.end(ctx, frame)?;

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
