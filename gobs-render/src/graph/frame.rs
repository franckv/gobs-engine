use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use gobs_core::{utils::timer::Timer, ImageExtent2D, ImageFormat};
use gobs_gfx::{Command, Device, Display, Image, ImageLayout, ImageUsage};

use crate::{
    batch::RenderBatch,
    context::Context,
    pass::{
        bounds::BoundsPass, compute::ComputePass, depth::DepthPass, forward::ForwardPass,
        ui::UiPass, wire::WirePass, PassId, PassType, RenderPass,
    },
    stats::RenderStats,
    GfxCommand, GfxImage,
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1920;

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Error,
}

pub struct FrameData {
    pub command: GfxCommand,
    //TODO: pub query_pool: QueryPool,
}

impl FrameData {
    pub fn new(ctx: &Context) -> Self {
        let command = GfxCommand::new(&ctx.device, "Frame");

        //TODO: let query_pool = QueryPool::new(ctx.device.clone(), QueryType::Timestamp, 2);

        FrameData { command }
    }

    pub fn reset(&mut self) {
        self.command.reset();
    }
}

pub struct ResourceManager {
    resources: HashMap<String, RwLock<GfxImage>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn register_image(
        &mut self,
        ctx: &Context,
        label: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) {
        let image = GfxImage::new(label, &ctx.device, format, usage, extent);

        self.resources.insert(label.to_string(), RwLock::new(image));
    }

    pub fn invalidate(&self) {
        for (_, image) in &self.resources {
            image.write().invalidate();
        }
    }

    pub fn image_read(&self, label: &str) -> RwLockReadGuard<'_, GfxImage> {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {}",
            label
        );

        self.resources[label].read()
    }

    pub fn image_write(&self, label: &str) -> RwLockWriteGuard<'_, GfxImage> {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {}",
            label
        );

        self.resources[label].write()
    }
}

pub struct FrameGraph {
    pub frames: Vec<FrameData>,
    pub draw_extent: ImageExtent2D,
    pub render_scaling: f32,
    pub passes: Vec<Arc<dyn RenderPass>>,
    resource_manager: ResourceManager,
    pub batch: RenderBatch,
}

impl FrameGraph {
    pub fn new(ctx: &Context) -> Self {
        let draw_extent = ctx.extent();

        let frames = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx))
            .collect();

        Self {
            frames,
            draw_extent,
            render_scaling: 1.,
            passes: Vec::new(),
            resource_manager: ResourceManager::new(),
            batch: RenderBatch::new(ctx),
        }
    }

    pub fn default(ctx: &Context) -> Self {
        let mut graph = Self::new(ctx);

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

        graph.register_pass(ComputePass::new(ctx, "compute"));
        graph.register_pass(DepthPass::new(ctx, "depth"));
        graph.register_pass(ForwardPass::new(ctx, "forward", false, false));
        graph.register_pass(UiPass::new(ctx, "ui", false));
        graph.register_pass(WirePass::new(ctx, "wire"));
        graph.register_pass(BoundsPass::new(ctx, "bounds"));

        graph
    }

    pub fn ui(ctx: &Context) -> Self {
        let mut graph = Self::new(ctx);

        let extent = Self::get_render_target_extent(ctx);

        graph.resource_manager.register_image(
            ctx,
            "draw",
            ctx.color_format,
            ImageUsage::Color,
            extent,
        );

        graph.register_pass(UiPass::new(ctx, "ui", true));

        graph
    }

    fn get_render_target_extent(ctx: &Context) -> ImageExtent2D {
        let extent = ctx.extent();
        ImageExtent2D::new(
            extent.width.max(FRAME_WIDTH),
            extent.height.max(FRAME_HEIGHT),
        )
    }

    fn register_pass(&mut self, pass: Arc<dyn RenderPass>) {
        for attach in pass.attachments() {
            assert!(self.resource_manager.resources.contains_key(attach));
        }

        self.passes.push(pass);
    }

    pub fn get_pass<F>(&self, cmp: F) -> Result<Arc<dyn RenderPass>, ()>
    where
        F: Fn(&Arc<dyn RenderPass>) -> bool,
    {
        for pass in &self.passes {
            if cmp(pass) {
                return Ok(pass.clone());
            }
        }

        Err(())
    }

    pub fn pass_by_id(&self, pass_id: PassId) -> Result<Arc<dyn RenderPass>, ()> {
        self.get_pass(|pass| pass.id() == pass_id)
    }

    pub fn pass_by_type(&self, pass_type: PassType) -> Result<Arc<dyn RenderPass>, ()> {
        self.get_pass(|pass| pass.ty() == pass_type)
    }

    pub fn pass_by_name(&self, pass_name: &str) -> Result<Arc<dyn RenderPass>, ()> {
        self.get_pass(|pass| pass.name() == pass_name)
    }

    pub fn render_stats(&self) -> &RenderStats {
        &self.batch.render_stats
    }

    pub fn begin(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        log::debug!("Begin new frame");

        let frame_id = ctx.frame_id();
        let frame = &mut self.frames[frame_id];
        frame.reset();

        self.batch.reset(ctx);

        let cmd = &frame.command;

        if ctx.frame_number >= ctx.frames_in_flight && ctx.frame_number % ctx.stats_refresh == 0 {
            //TODO: let mut buf = [0 as u64; 2];
            //frame.query_pool.get_query_pool_results(0, &mut buf);

            //self.batch.render_stats.gpu_draw_time =
            //    ((buf[1] - buf[0]) as f32 * frame.query_pool.period) / 1_000_000_000.;
        }

        let draw_image_extent = self.resource_manager.image_read("draw").extent();
        if self.resource_manager.resources.contains_key("depth") {
            debug_assert_eq!(
                draw_image_extent,
                self.resource_manager.image_read("depth").extent()
            );
        }

        let display_extent = ctx.extent();

        self.draw_extent = ImageExtent2D::new(
            (draw_image_extent.width.min(display_extent.width) as f32 * self.render_scaling) as u32,
            (draw_image_extent.height.min(display_extent.height) as f32 * self.render_scaling)
                as u32,
        );

        if let Err(_) = ctx.display.acquire(ctx.frame_id()) {
            return Err(RenderError::Outdated);
        }

        self.resource_manager.invalidate();

        cmd.begin();

        cmd.begin_label(&format!("Frame {}", ctx.frame_number));

        //TODO: cmd.reset_query_pool(&frame.query_pool, 0, 2);
        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::TopOfPipe, 0);

        Ok(())
    }

    pub fn end(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        log::debug!("End frame");

        let frame_id = ctx.frame_id();
        let frame = &self.frames[frame_id];
        let cmd = &frame.command;

        log::debug!("Present");

        cmd.transition_image_layout(
            &mut self.resource_manager.image_write("draw"),
            ImageLayout::TransferSrc,
        );

        let render_target = ctx.display.get_render_target();

        cmd.transition_image_layout(render_target, ImageLayout::TransferDst);

        cmd.copy_image_to_image(
            &self.resource_manager.image_read("draw"),
            self.draw_extent,
            render_target,
            render_target.extent(),
        );

        cmd.transition_image_layout(render_target, ImageLayout::Present);

        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::BottomOfPipe, 1);

        cmd.end_label();

        cmd.end();

        cmd.submit2(&ctx.display, ctx.frame_id());

        let Ok(_) = ctx.display.present(&ctx.device, ctx.frame_id()) else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    pub fn update(&mut self, ctx: &Context, delta: f32) {
        if ctx.frame_number % ctx.stats_refresh == 0 {
            self.batch.render_stats.fps = (1. / delta).round() as u32;
        }
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        draw_cmd: &mut dyn FnMut(Arc<dyn RenderPass>, &mut RenderBatch),
    ) -> Result<(), RenderError> {
        log::debug!("Begin rendering");

        let frame_id = ctx.frame_id();

        let mut timer = Timer::new();

        let should_update = ctx.frame_number % ctx.stats_refresh == 0;

        self.batch.render_stats.update_time_reset(should_update);
        for pass in &self.passes {
            draw_cmd(pass.clone(), &mut self.batch);

            self.batch
                .render_stats
                .update_time_add(timer.delta(), pass.id(), should_update);
        }

        self.batch.finish();

        let cmd = &self.frames[frame_id].command;

        self.batch.render_stats.cpu_draw_time_reset(should_update);
        for pass in &self.passes {
            log::debug!("Enter render pass: {}", pass.name());
            pass.render(
                ctx,
                cmd,
                &self.resource_manager,
                &mut self.batch,
                self.draw_extent,
            )?;

            self.batch
                .render_stats
                .cpu_draw_time_add(timer.delta(), pass.id(), should_update);
        }

        log::debug!("End rendering");

        Ok(())
    }

    pub fn resize(&mut self, ctx: &mut Context) {
        self.resize_swapchain(ctx);
    }

    fn resize_swapchain(&mut self, ctx: &mut Context) {
        ctx.device.wait();

        ctx.display.resize(&ctx.device);
    }
}
