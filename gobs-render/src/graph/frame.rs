use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use gobs_utils::timer::Timer;
use gobs_vulkan::{
    command::{CommandBuffer, CommandPool},
    image::{ColorSpace, Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage},
    pipeline::PipelineStage,
    query::{QueryPool, QueryType},
    swapchain::{PresentationMode, SwapChain},
    sync::Semaphore,
};

use crate::{
    context::Context,
    pass::{
        compute::ComputePass, forward::ForwardPass, ui::UiPass, wire::WirePass, PassId, PassType,
        RenderPass,
    },
    renderable::RenderBatch,
    stats::RenderStats,
};

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Error,
}

pub struct FrameData {
    pub command_buffer: CommandBuffer,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
    pub query_pool: QueryPool,
}

impl FrameData {
    pub fn new(ctx: &Context) -> Self {
        let command_pool = CommandPool::new(ctx.device.clone(), &ctx.queue.family);
        let command_buffer =
            CommandBuffer::new(ctx.device.clone(), ctx.queue.clone(), command_pool, "Frame");

        let swapchain_semaphore = Semaphore::new(ctx.device.clone(), "Swapchain");
        let render_semaphore = Semaphore::new(ctx.device.clone(), "Render");

        let query_pool = QueryPool::new(ctx.device.clone(), QueryType::Timestamp, 2);

        FrameData {
            command_buffer,
            swapchain_semaphore,
            render_semaphore,
            query_pool,
        }
    }

    pub fn reset(&mut self) {
        self.command_buffer.reset();
    }
}

pub struct ResourceManager {
    resources: HashMap<String, RwLock<Image>>,
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
        let image = Image::new(
            label,
            ctx.device.clone(),
            format,
            usage,
            extent,
            ctx.allocator.clone(),
        );

        self.resources.insert(label.to_string(), RwLock::new(image));
    }

    pub fn invalidate(&self) {
        for (_, image) in &self.resources {
            image.write().invalidate();
        }
    }

    pub fn image_read(&self, label: &str) -> RwLockReadGuard<'_, Image> {
        self.resources[label].read()
    }

    pub fn image_write(&self, label: &str) -> RwLockWriteGuard<'_, Image> {
        self.resources[label].write()
    }
}

pub struct FrameGraph {
    pub frame_number: usize,
    pub frames: Vec<FrameData>,
    pub swapchain: SwapChain,
    pub swapchain_images: Vec<Image>,
    pub swapchain_idx: usize,
    pub draw_extent: ImageExtent2D,
    pub render_scaling: f32,
    pub passes: Vec<Arc<dyn RenderPass>>,
    resource_manager: ResourceManager,
    pub batch: RenderBatch,
}

impl FrameGraph {
    pub fn new(ctx: &Context) -> Self {
        let swapchain = Self::create_swapchain(ctx);
        let swapchain_images = swapchain.create_images();

        let draw_extent = ctx.surface.get_extent(ctx.device.clone());

        let frames = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx))
            .collect();

        Self {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            swapchain_idx: 0,
            draw_extent,
            render_scaling: 1.,
            passes: Vec::new(),
            resource_manager: ResourceManager::new(),
            batch: RenderBatch::new(),
        }
    }

    pub fn default(ctx: &Context) -> Self {
        let mut graph = Self::new(ctx);

        let extent = ctx.surface.get_extent(ctx.device.clone());

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
        graph.register_pass(ForwardPass::new(ctx, "forward"));
        graph.register_pass(UiPass::new(ctx, "ui"));
        graph.register_pass(WirePass::new(ctx, "wire"));

        graph
    }

    fn register_pass(&mut self, pass: Arc<dyn RenderPass>) {
        for attach in pass.attachments() {
            assert!(self.resource_manager.resources.contains_key(attach));
        }

        self.passes.push(pass);
    }

    fn new_frame(&mut self, ctx: &Context) -> usize {
        self.frame_number += 1;
        (self.frame_number - 1) % ctx.frames_in_flight
    }

    pub fn frame_id(&self, ctx: &Context) -> usize {
        (self.frame_number - 1) % ctx.frames_in_flight
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

    pub fn begin(&mut self, ctx: &Context) -> Result<(), RenderError> {
        let frame_id = self.new_frame(ctx);
        self.batch.reset();

        {
            let frame = &mut self.frames[frame_id];
            let cmd = &frame.command_buffer;

            cmd.fence.wait_and_reset();
            debug_assert!(!cmd.fence.signaled());
            frame.reset();
        }

        let frame = &self.frames[frame_id];
        let cmd = &frame.command_buffer;

        if self.frame_number >= ctx.frames_in_flight && self.frame_number % ctx.stats_refresh == 0 {
            let mut buf = [0 as u64; 2];
            frame.query_pool.get_query_pool_results(0, 2, &mut buf);

            self.batch.render_stats.gpu_draw_time =
                ((buf[1] - buf[0]) as f32 * frame.query_pool.period) / 1_000_000_000.;
        }

        let draw_image_extent = self.resource_manager.image_read("draw").extent;
        debug_assert_eq!(
            draw_image_extent,
            self.resource_manager.image_read("depth").extent
        );

        self.draw_extent = ImageExtent2D::new(
            (draw_image_extent
                .width
                .min(ctx.surface.get_dimensions().width) as f32
                * self.render_scaling) as u32,
            (draw_image_extent
                .height
                .min(ctx.surface.get_dimensions().height) as f32
                * self.render_scaling) as u32,
        );

        let Ok(image_index) = self.swapchain.acquire_image(&frame.swapchain_semaphore) else {
            return Err(RenderError::Outdated);
        };

        self.swapchain_idx = image_index;

        self.resource_manager.invalidate();
        self.swapchain_images[image_index as usize].invalidate();

        cmd.begin();

        cmd.begin_label(&format!("Frame {}", self.frame_number));

        cmd.reset_query_pool(&frame.query_pool, 0, 2);
        cmd.write_timestamp(&frame.query_pool, PipelineStage::TopOfPipe, 0);

        Ok(())
    }

    pub fn end(&mut self, ctx: &Context) -> Result<(), RenderError> {
        let frame_id = self.frame_id(ctx);
        let frame = &self.frames[frame_id];
        let cmd = &frame.command_buffer;

        log::debug!("Present");

        cmd.transition_image_layout(
            &mut self.resource_manager.image_write("draw"),
            ImageLayout::TransferSrc,
        );

        let swapchain_image = &mut self.swapchain_images[self.swapchain_idx];

        cmd.transition_image_layout(swapchain_image, ImageLayout::TransferDst);

        cmd.copy_image_to_image(
            &self.resource_manager.image_read("draw"),
            self.draw_extent,
            swapchain_image,
            swapchain_image.extent,
        );

        cmd.transition_image_layout(swapchain_image, ImageLayout::Present);

        cmd.write_timestamp(&frame.query_pool, PipelineStage::BottomOfPipe, 1);

        cmd.end_label();

        cmd.end();

        cmd.submit2(
            Some(&frame.swapchain_semaphore),
            Some(&frame.render_semaphore),
        );

        let Ok(_) = self
            .swapchain
            .present(self.swapchain_idx, &ctx.queue, &frame.render_semaphore)
        else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        draw_cmd: &mut dyn FnMut(Arc<dyn RenderPass>, &mut RenderBatch),
    ) -> Result<(), RenderError> {
        log::debug!("Begin rendering");

        let frame_id = self.frame_id(ctx);

        let mut timer = Timer::new();

        for pass in &self.passes {
            draw_cmd(pass.clone(), &mut self.batch);
        }

        self.batch.finish();

        if self.frame_number % ctx.stats_refresh == 0 {
            self.batch.render_stats.update_time = timer.delta();
        }

        let cmd = &self.frames[frame_id].command_buffer;

        for pass in &self.passes {
            pass.render(
                ctx,
                cmd,
                &self.resource_manager,
                &mut self.batch,
                self.draw_extent,
            )?;
        }

        if self.frame_number % ctx.stats_refresh == 0 {
            self.batch.render_stats.cpu_draw_time = timer.peek();
        }

        log::debug!("End rendering");

        Ok(())
    }

    pub fn resize(&mut self, ctx: &Context) {
        self.resize_swapchain(ctx);
    }

    fn create_swapchain(ctx: &Context) -> SwapChain {
        let device = ctx.device.clone();
        let surface = ctx.surface.clone();

        let presents = surface.get_available_presentation_modes(device.clone());

        let present = *presents
            .iter()
            .find(|&&p| p == PresentationMode::Fifo)
            .unwrap();

        let caps = surface.get_capabilities(device.clone());

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let formats = surface.get_available_format(&device.p_device);

        let format = *formats
            .iter()
            .find(|f| {
                f.format == ImageFormat::B8g8r8a8Unorm && f.color_space == ColorSpace::SrgbNonlinear
            })
            .unwrap();

        log::info!("Swapchain format: {:?}", format);

        SwapChain::new(device, surface, format, present, image_count, None)
    }

    fn resize_swapchain(&mut self, ctx: &Context) {
        ctx.device.wait();

        self.swapchain = SwapChain::new(
            ctx.device.clone(),
            ctx.surface.clone(),
            self.swapchain.format,
            self.swapchain.present,
            self.swapchain.image_count,
            Some(&self.swapchain),
        );
        self.swapchain_images = self.swapchain.create_images();
    }
}
