use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{ImageLayout, ImageUsage, SceneData};
use gobs_resource::ResourceManager;

use crate::{
    FrameData, GfxContext, GraphConfig, PassId, PipelinesConfig, RenderError, RenderObject,
    RenderPass,
    graph::resource::GraphResourceManager,
    pass::{PassType, compute::ComputePass, present::PresentPass},
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

pub struct FrameGraphPass {
    pub pass: RenderPass,
    pub enabled: bool,
}

pub struct FrameGraph {
    pub render_scaling: f32,
    pub passes: Vec<FrameGraphPass>,
    resource_manager: GraphResourceManager,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self {
            render_scaling: 1.,
            passes: Vec::new(),
            resource_manager: GraphResourceManager::new(),
        }
    }

    pub fn standard(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        graph_name: &str,
    ) -> Result<Self, RenderError> {
        let mut graph = Self::new();

        let extent = Self::get_render_target_extent(ctx);

        graph.resource_manager.register_image(
            ctx,
            "draw",
            ctx.color_format,
            ImageUsage::Color,
            extent,
        );
        graph.resource_manager.register_image(
            ctx,
            "depth",
            ctx.depth_format,
            ImageUsage::Depth,
            extent,
        );

        graph.register_pass(ComputePass::new(ctx, "compute")?);

        PipelinesConfig::load_resources(ctx, "pipelines.ron", resource_manager)
            .expect("Load pipelines");

        tracing::debug!(target: logger::INIT, "Load graph: {}", "scene");
        let passes = GraphConfig::load_graph(ctx, "graph.ron", graph_name, resource_manager)
            .map_err(|_| RenderError::InvalidData)?;
        for pass in &passes {
            tracing::debug!(target: logger::INIT, "Load pass: {}", pass.name());
            graph.register_pass(pass.clone());
        }

        graph.register_pass(PresentPass::new(ctx, "present")?);

        Ok(graph)
    }

    pub fn headless(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, RenderError> {
        let mut graph = Self::new();

        let extent = Self::get_render_target_extent(ctx);

        graph.resource_manager.register_image(
            ctx,
            "draw",
            ctx.color_format,
            ImageUsage::Color,
            extent,
        );
        graph.resource_manager.register_image(
            ctx,
            "depth",
            ctx.depth_format,
            ImageUsage::Depth,
            extent,
        );

        PipelinesConfig::load_resources(ctx, "pipelines.ron", resource_manager)
            .expect("Load pipelines");

        let passes = GraphConfig::load_graph(ctx, "graph.ron", "headless", resource_manager)
            .map_err(|_| RenderError::InvalidData)?;

        graph.register_pass(ComputePass::new(ctx, "compute")?);

        for pass in &passes {
            graph.register_pass(pass.clone());
        }

        Ok(graph)
    }

    pub fn ui(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, RenderError> {
        let mut graph = Self::new();

        let extent = Self::get_render_target_extent(ctx);

        graph.resource_manager.register_image(
            ctx,
            "draw",
            ctx.color_format,
            ImageUsage::Color,
            extent,
        );

        PipelinesConfig::load_resources(ctx, "pipelines.ron", resource_manager)
            .expect("Load pipelines");

        let passes = GraphConfig::load_graph(ctx, "graph.ron", "ui", resource_manager)
            .map_err(|_| RenderError::InvalidData)?;

        for pass in &passes {
            graph.register_pass(pass.clone());
        }

        graph.register_pass(PresentPass::new(ctx, "present")?);

        Ok(graph)
    }

    fn get_render_target_extent(ctx: &GfxContext) -> ImageExtent2D {
        let extent = ctx.extent();
        ImageExtent2D::new(
            extent.width.max(FRAME_WIDTH),
            extent.height.max(FRAME_HEIGHT),
        )
    }

    pub fn register_pass(&mut self, pass: RenderPass) {
        let pass = FrameGraphPass {
            pass,
            enabled: true,
        };

        self.passes.push(pass);
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

        let draw_image_extent = ctx
            .hal
            .get_image_extent(self.resource_manager.image("draw"));
        if self.resource_manager.resources.contains_key("depth") {
            debug_assert_eq!(
                draw_image_extent,
                ctx.hal
                    .get_image_extent(self.resource_manager.image("depth"))
            );
        }

        if ctx.hal.acquire(frame.id).is_err() {
            return Err(RenderError::Outdated);
        }

        self.resource_manager.invalidate(ctx.hal.as_mut());

        cmd.begin(frame.frame_number);

        cmd.begin_label(&format!("Frame {}", frame.frame_number));

        //TODO: cmd.reset_query_pool(&frame.query_pool, 0, 2);
        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::TopOfPipe, 0);

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn end(&mut self, ctx: &mut GfxContext, frame: &mut FrameData) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "End frame");

        let frame_id = frame.frame_number % ctx.frames_in_flight;
        let cmd = &frame.command;

        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::BottomOfPipe, 1);

        let render_target = ctx.hal.get_render_target();
        cmd.transition_image_layout(ctx.hal.as_mut(), render_target, ImageLayout::Present);

        cmd.end_label();

        cmd.end();

        cmd.submit2(ctx.hal.as_ref(), frame_id);
        frame.submitted = true;

        let Ok(_) = ctx.hal.present() else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    pub fn update(&mut self, _ctx: &GfxContext, _delta: f32) {}

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn render(
        &mut self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        render_list: &[RenderObject],
        scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        for pass in &mut self.passes {
            if !pass.enabled {
                continue;
            }

            let pass = &pass.pass;
            let span =
                tracing::span!(target: logger::PROFILE, tracing::Level::TRACE, "Pass", "{}", pass.name())
                    .entered();
            tracing::debug!(target: logger::RENDER, "Begin rendering pass {}", pass.name());

            pass.render(ctx, frame, &self.resource_manager, render_list, scene_data)?;

            tracing::debug!(target: logger::RENDER, "End rendering pass");
            span.exit();
        }

        Ok(())
    }

    pub fn resize(&mut self, ctx: &mut GfxContext) {
        self.resize_swapchain(ctx);
    }

    fn resize_swapchain(&mut self, ctx: &mut GfxContext) {
        ctx.hal.wait();

        ctx.hal.resize();
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
