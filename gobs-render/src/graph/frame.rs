use bytemuck::Pod;

use gobs_core::{ImageExtent2D, ImageFormat, utils::timer::Timer};
use gobs_gfx::{
    Buffer, BufferUsage, Command, CommandQueueType, Device, Display, GfxBuffer, GfxCommand,
    GfxImage, Image, ImageLayout, ImageUsage,
};

use crate::{
    GfxContext, RenderError, RenderPass,
    batch::RenderBatch,
    graph::resource::GraphResourceManager,
    pass::{
        PassId, PassType, bounds::BoundsPass, compute::ComputePass, depth::DepthPass,
        dummy::DummyPass, forward::ForwardPass, present::PresentPass, ui::UiPass, wire::WirePass,
    },
    stats::RenderStats,
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

pub struct FrameData {
    pub id: usize,
    pub command: GfxCommand,
    //TODO: pub query_pool: QueryPool,
}

impl FrameData {
    pub fn new(ctx: &GfxContext, id: usize) -> Self {
        let command = GfxCommand::new(&ctx.device, "Frame", CommandQueueType::Graphics);

        //TODO: let query_pool = QueryPool::new(ctx.device.clone(), QueryType::Timestamp, 2);

        FrameData { id, command }
    }

    pub fn reset(&mut self) {
        self.command.reset();
    }
}

pub struct FrameGraph {
    pub frames: Vec<FrameData>,
    pub frame_number: usize,
    pub draw_extent: ImageExtent2D,
    pub render_scaling: f32,
    pub passes: Vec<RenderPass>,
    resource_manager: GraphResourceManager,
    pub batch: RenderBatch,
}

impl FrameGraph {
    pub fn new(ctx: &GfxContext) -> Self {
        let draw_extent = ctx.extent();

        let frames = (0..ctx.frames_in_flight)
            .map(|id| FrameData::new(ctx, id))
            .collect();

        Self {
            frames,
            frame_number: 0,
            draw_extent,
            render_scaling: 1.,
            passes: Vec::new(),
            resource_manager: GraphResourceManager::new(),
            batch: RenderBatch::new(),
        }
    }

    pub fn default(ctx: &GfxContext) -> Result<Self, RenderError> {
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

        graph.register_pass(ComputePass::new(ctx, "compute")?);
        graph.register_pass(DepthPass::new(ctx, "depth")?);
        graph.register_pass(ForwardPass::new(ctx, "forward", false, false)?);
        graph.register_pass(UiPass::new(ctx, "ui", false)?);
        graph.register_pass(WirePass::new(ctx, "wire")?);
        graph.register_pass(BoundsPass::new(ctx, "bounds")?);
        graph.register_pass(DummyPass::new(ctx, "dummy")?);
        graph.register_pass(PresentPass::new(ctx, "present")?);

        Ok(graph)
    }

    pub fn headless(ctx: &GfxContext) -> Result<Self, RenderError> {
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

        graph.register_pass(ComputePass::new(ctx, "compute")?);
        graph.register_pass(DepthPass::new(ctx, "depth")?);
        graph.register_pass(ForwardPass::new(ctx, "forward", false, false)?);
        graph.register_pass(DummyPass::new(ctx, "dummy")?);

        Ok(graph)
    }

    pub fn ui(ctx: &GfxContext) -> Result<Self, RenderError> {
        let mut graph = Self::new(ctx);

        let extent = Self::get_render_target_extent(ctx);

        graph.resource_manager.register_image(
            ctx,
            "draw",
            ctx.color_format,
            ImageUsage::Color,
            extent,
        );

        graph.register_pass(UiPass::new(ctx, "ui", true)?);
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

    fn register_pass(&mut self, pass: RenderPass) {
        for attach in pass.attachments() {
            assert!(self.resource_manager.resources.contains_key(attach));
        }

        self.passes.push(pass);
    }

    pub fn get_pass<F>(&self, cmp: F) -> Result<RenderPass, RenderError>
    where
        F: Fn(&RenderPass) -> bool,
    {
        for pass in &self.passes {
            if cmp(pass) {
                return Ok(pass.clone());
            }
        }

        Err(RenderError::PassNotFound)
    }

    pub fn get_image_data<T: Pod>(
        &self,
        ctx: &GfxContext,
        label: &str,
        data: &mut Vec<T>,
        format: ImageFormat,
    ) -> ImageExtent2D {
        ctx.device.wait();

        let mut src_image = self.resource_manager.image_write(label);
        let mut mid_image = GfxImage::new(
            "mid",
            &ctx.device,
            format,
            ImageUsage::Color,
            src_image.extent(),
        );
        let mut dst_image = GfxImage::new(
            "dst",
            &ctx.device,
            format,
            ImageUsage::File,
            src_image.extent(),
        );

        let mut buffer = GfxBuffer::new(
            "copy",
            dst_image.size(),
            BufferUsage::StagingDst,
            &ctx.device,
        );

        ctx.device.run_immediate_mut(|cmd| {
            cmd.transition_image_layout(&mut src_image, ImageLayout::TransferSrc);
            cmd.transition_image_layout(&mut mid_image, ImageLayout::TransferDst);
            let dst_extent = mid_image.extent();
            cmd.copy_image_to_image(&src_image, src_image.extent(), &mut mid_image, dst_extent);

            cmd.transition_image_layout(&mut mid_image, ImageLayout::TransferSrc);
            cmd.transition_image_layout(&mut dst_image, ImageLayout::TransferDst);
            let dst_extent = dst_image.extent();
            cmd.copy_image_to_image(&mid_image, mid_image.extent(), &mut dst_image, dst_extent);

            cmd.transition_image_layout(&mut dst_image, ImageLayout::TransferSrc);
            cmd.copy_image_to_buffer(&dst_image, &mut buffer);
        });

        buffer.get_bytes(data);

        dst_image.extent()
    }

    pub fn pass_by_id(&self, pass_id: PassId) -> Result<RenderPass, RenderError> {
        self.get_pass(|pass| pass.id() == pass_id)
    }

    pub fn pass_by_type(&self, pass_type: PassType) -> Result<RenderPass, RenderError> {
        self.get_pass(|pass| pass.ty() == pass_type)
    }

    pub fn pass_by_name(&self, pass_name: &str) -> Result<RenderPass, RenderError> {
        self.get_pass(|pass| pass.name() == pass_name)
    }

    pub fn render_stats(&self) -> &RenderStats {
        &self.batch.render_stats
    }

    pub fn begin(&mut self, ctx: &mut GfxContext) -> Result<(), RenderError> {
        self.frame_number += 1;
        let frame_id = self.frame_number % ctx.frames_in_flight;

        tracing::debug!(target: "render", "Begin new frame: {} ({}/{})", self.frame_number, frame_id, ctx.frames_in_flight);

        let frame = &mut self.frames[frame_id];
        frame.reset();

        self.batch.reset(ctx);

        let cmd = &frame.command;

        if self.frame_number >= ctx.frames_in_flight && self.frame_number % ctx.stats_refresh == 0 {
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

        self.draw_extent = ImageExtent2D::new(
            (draw_image_extent.width as f32 * self.render_scaling) as u32,
            (draw_image_extent.height as f32 * self.render_scaling) as u32,
        );

        tracing::trace!(target: "render", "Draw extent {:?}", self.draw_extent);

        if ctx.display.acquire(frame_id).is_err() {
            return Err(RenderError::Outdated);
        }

        self.resource_manager.invalidate();

        cmd.begin();

        cmd.begin_label(&format!("Frame {}", self.frame_number));

        //TODO: cmd.reset_query_pool(&frame.query_pool, 0, 2);
        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::TopOfPipe, 0);

        Ok(())
    }

    pub fn end(&mut self, ctx: &mut GfxContext) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "End frame");

        let frame_id = self.frame_number % ctx.frames_in_flight;
        let frame = &self.frames[frame_id];
        let cmd = &frame.command;

        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::BottomOfPipe, 1);

        if let Some(render_target) = ctx.display.get_render_target() {
            cmd.transition_image_layout(render_target, ImageLayout::Present);
        }

        cmd.end_label();

        cmd.end();

        ctx.device.wait_transfer();

        cmd.submit2(&ctx.display, frame_id);

        let Ok(_) = ctx.display.present(&ctx.device, frame_id) else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    pub fn update(&mut self, ctx: &GfxContext, delta: f32) {
        if self.frame_number % ctx.stats_refresh == 0 {
            self.batch.render_stats.fps = (1. / delta).round() as u32;
        }
    }

    pub fn prepare(
        &mut self,
        ctx: &GfxContext,
        draw_cmd: &mut dyn FnMut(RenderPass, &mut RenderBatch),
    ) {
        tracing::debug!(target: "render", "Begin render batch");

        let mut timer = Timer::new();

        let should_update = self.frame_number % ctx.stats_refresh == 0;

        self.batch.render_stats.update_time_reset(should_update);
        for pass in &self.passes {
            draw_cmd(pass.clone(), &mut self.batch);

            self.batch
                .render_stats
                .update_time_add(timer.delta(), pass.id(), should_update);
        }

        self.batch.finish();
    }

    pub fn render(&mut self, ctx: &mut GfxContext) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Begin rendering");

        let frame_id = self.frame_number % ctx.frames_in_flight;

        let mut timer = Timer::new();

        let should_update = self.frame_number % ctx.stats_refresh == 0;

        self.batch.render_stats.cpu_draw_time_reset(should_update);

        let frame = &self.frames[frame_id];

        for pass in &self.passes {
            tracing::debug!(target: "render", "Enter render pass: {}", pass.name());
            pass.render(
                ctx,
                frame,
                &self.resource_manager,
                &mut self.batch,
                self.draw_extent,
            )?;

            self.batch
                .render_stats
                .cpu_draw_time_add(timer.delta(), pass.id(), should_update);
        }

        tracing::debug!(target: "render", "End rendering");

        Ok(())
    }

    pub fn resize(&mut self, ctx: &mut GfxContext) {
        self.resize_swapchain(ctx);
    }

    fn resize_swapchain(&mut self, ctx: &mut GfxContext) {
        ctx.device.wait();

        ctx.display.resize(&ctx.device);
    }
}
