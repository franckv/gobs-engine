use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

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
        compute::ComputePass, forward::ForwardPass, ui::UiPass, wire::WirePass, PassId, RenderPass,
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

pub struct FrameGraph {
    pub frame_number: usize,
    pub frames: Vec<FrameData>,
    pub swapchain: SwapChain,
    pub swapchain_images: Vec<Image>,
    pub swapchain_idx: usize,
    pub draw_extent: ImageExtent2D,
    pub render_scaling: f32,
    pub passes: HashMap<String, Arc<dyn RenderPass>>,
    resource_manager: HashMap<String, RwLock<Image>>,
    pub batch: RenderBatch,
}

impl FrameGraph {
    pub fn new(ctx: &Context) -> Self {
        let swapchain = Self::create_swapchain(ctx);
        let swapchain_images = swapchain.create_images();

        let extent = ctx.surface.get_extent(ctx.device.clone());

        let draw_image = Image::new(
            "color",
            ctx.device.clone(),
            ctx.color_format,
            ImageUsage::Color,
            extent,
            ctx.allocator.clone(),
        );

        let depth_image = Image::new(
            "depth",
            ctx.device.clone(),
            ctx.depth_format,
            ImageUsage::Depth,
            extent,
            ctx.allocator.clone(),
        );

        let mut resource_manager = HashMap::new();

        resource_manager.insert("draw".to_string(), RwLock::new(draw_image));
        resource_manager.insert("depth".to_string(), RwLock::new(depth_image));

        let frames = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx))
            .collect();

        let mut passes = HashMap::new();

        passes.insert(
            "compute".to_string(),
            ComputePass::new(ctx, "bg", &resource_manager["draw"].read()),
        );
        passes.insert("forward".to_string(), ForwardPass::new(ctx, "scene"));
        passes.insert("ui".to_string(), UiPass::new(ctx, "ui"));
        passes.insert("wire".to_string(), WirePass::new(ctx, "wire"));

        Self {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            swapchain_idx: 0,
            draw_extent: extent,
            render_scaling: 1.,
            passes,
            resource_manager,
            batch: RenderBatch::new(),
        }
    }

    fn new_frame(&mut self, ctx: &Context) -> usize {
        self.frame_number += 1;
        (self.frame_number - 1) % ctx.frames_in_flight
    }

    pub fn frame_id(&self, ctx: &Context) -> usize {
        (self.frame_number - 1) % ctx.frames_in_flight
    }

    pub fn pass(&self, pass_id: PassId) -> Arc<dyn RenderPass> {
        for (_, pass) in &self.passes {
            if pass.id() == pass_id {
                return pass.clone();
            }
        }

        return self.passes["forward"].clone();
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

        let draw_image = &self.resource_manager["draw"];
        let depth_image = &self.resource_manager["draw"];

        let draw_image_extent = draw_image.read().extent;
        debug_assert_eq!(draw_image_extent, depth_image.read().extent);

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

        draw_image.write().invalidate();
        depth_image.write().invalidate();
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

        let draw_image = &self.resource_manager["draw"];
        cmd.transition_image_layout(&mut draw_image.write(), ImageLayout::TransferSrc);

        let swapchain_image = &mut self.swapchain_images[self.swapchain_idx];

        cmd.transition_image_layout(swapchain_image, ImageLayout::TransferDst);

        cmd.copy_image_to_image(
            &draw_image.read(),
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

        draw_cmd(self.passes["compute"].clone(), &mut self.batch);
        draw_cmd(self.passes["forward"].clone(), &mut self.batch);
        draw_cmd(self.passes["wire"].clone(), &mut self.batch);
        draw_cmd(self.passes["ui"].clone(), &mut self.batch);

        self.batch.finish();

        self.batch.render_stats.update_time = timer.delta();

        let cmd = &self.frames[frame_id].command_buffer;

        let draw_image = &self.resource_manager["draw"];
        let depth_image = &self.resource_manager["depth"];

        self.passes["compute"].render(
            ctx,
            cmd,
            &mut [&mut draw_image.write()],
            &mut self.batch,
            self.draw_extent,
        )?;

        self.passes["forward"].render(
            ctx,
            cmd,
            &mut [&mut draw_image.write(), &mut depth_image.write()],
            &mut self.batch,
            self.draw_extent,
        )?;

        self.passes["wire"].render(
            ctx,
            cmd,
            &mut [&mut draw_image.write()],
            &mut self.batch,
            self.draw_extent,
        )?;

        self.passes["ui"].render(
            ctx,
            cmd,
            &mut [&mut draw_image.write()],
            &mut self.batch,
            self.draw_extent,
        )?;

        self.batch.render_stats.cpu_draw_time = timer.peek();

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
