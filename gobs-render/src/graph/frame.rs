use std::sync::Arc;

use parking_lot::RwLock;

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
    pass::{compute::ComputePass, forward::ForwardPass, ui::UiPass, wire::WirePass, RenderPass},
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
    pub draw_image: RwLock<Image>,
    pub depth_image: RwLock<Image>,
    pub draw_extent: ImageExtent2D,
    pub render_scaling: f32,
    pub compute_pass: Arc<dyn RenderPass>,
    pub forward_pass: Arc<dyn RenderPass>,
    pub ui_pass: Arc<dyn RenderPass>,
    pub wire_pass: Arc<dyn RenderPass>,
    pub gpu_time: f32,
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

        let frames = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx))
            .collect();

        let compute_pass = ComputePass::new(ctx, "bg", &draw_image);
        let forward_pass = ForwardPass::new("scene");
        let ui_pass = UiPass::new("ui");
        let wire_pass = WirePass::new(ctx, "wire");

        Self {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            swapchain_idx: 0,
            draw_image: RwLock::new(draw_image),
            depth_image: RwLock::new(depth_image),
            draw_extent: extent,
            render_scaling: 1.,
            compute_pass,
            forward_pass,
            ui_pass,
            wire_pass,
            gpu_time: 0.,
        }
    }

    pub fn frame_id(&self, ctx: &Context) -> usize {
        self.frame_number % ctx.frames_in_flight
    }

    pub fn begin(&mut self, ctx: &Context) -> Result<(), RenderError> {
        let frame_id = self.frame_id(ctx);

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

            self.gpu_time = ((buf[1] - buf[0]) as f32 * frame.query_pool.period) / 1_000_000_000.;
        }

        let draw_image_extent = self.draw_image.read().extent;

        debug_assert_eq!(draw_image_extent, self.depth_image.read().extent);

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

        self.draw_image.write().invalidate();
        self.depth_image.write().invalidate();
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

        cmd.transition_image_layout(&mut self.draw_image.write(), ImageLayout::TransferSrc);

        let swapchain_image = &mut self.swapchain_images[self.swapchain_idx];

        cmd.transition_image_layout(swapchain_image, ImageLayout::TransferDst);

        cmd.copy_image_to_image(
            &self.draw_image.read(),
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

        self.frame_number += 1;

        Ok(())
    }

    pub fn render(
        &self,
        ctx: &Context,
        draw_cmd: &dyn Fn(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError> {
        log::debug!("Begin rendering");

        let frame_id = self.frame_id(ctx);
        let cmd = &self.frames[frame_id].command_buffer;

        self.compute_pass.clone().render(
            ctx,
            cmd,
            &mut [&mut self.draw_image.write()],
            self.draw_extent,
            draw_cmd,
        )?;

        self.forward_pass.clone().render(
            ctx,
            cmd,
            &mut [&mut self.draw_image.write(), &mut self.depth_image.write()],
            self.draw_extent,
            draw_cmd,
        )?;

        self.wire_pass.clone().render(
            ctx,
            cmd,
            &mut [&mut self.draw_image.write()],
            self.draw_extent,
            draw_cmd,
        )?;

        self.ui_pass.clone().render(
            ctx,
            cmd,
            &mut [&mut self.draw_image.write()],
            self.draw_extent,
            draw_cmd,
        )?;

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
