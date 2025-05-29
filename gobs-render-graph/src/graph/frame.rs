use bytemuck::Pod;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{
    Buffer, BufferUsage, Command, Device, Display, GfxBuffer, GfxImage, Image, ImageLayout,
    ImageUsage,
};

use crate::{
    FrameData, GfxContext, RenderError, RenderObject, RenderPass,
    graph::resource::GraphResourceManager,
    pass::{
        PassId, PassType, bounds::BoundsPass, compute::ComputePass, depth::DepthPass,
        dummy::DummyPass, forward::ForwardPass, present::PresentPass, ui::UiPass, wire::WirePass,
    },
};

const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

pub struct FrameGraph {
    pub draw_extent: ImageExtent2D,
    pub render_scaling: f32,
    pub passes: Vec<RenderPass>,
    resource_manager: GraphResourceManager,
}

impl FrameGraph {
    pub fn new(ctx: &GfxContext) -> Self {
        let draw_extent = ctx.extent();

        Self {
            draw_extent,
            render_scaling: 1.,
            passes: Vec::new(),
            resource_manager: GraphResourceManager::new(),
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

    pub fn begin(&mut self, ctx: &mut GfxContext, frame: &FrameData) -> Result<(), RenderError> {
        let cmd = &frame.command;

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

        if ctx.display.acquire(frame.id).is_err() {
            return Err(RenderError::Outdated);
        }

        self.resource_manager.invalidate();

        cmd.begin();

        cmd.begin_label(&format!("Frame {}", frame.frame_number));

        //TODO: cmd.reset_query_pool(&frame.query_pool, 0, 2);
        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::TopOfPipe, 0);

        Ok(())
    }

    pub fn end(&mut self, ctx: &mut GfxContext, frame: &FrameData) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "End frame");

        let frame_id = frame.frame_number % ctx.frames_in_flight;
        let cmd = &frame.command;

        //TODO: cmd.write_timestamp(&frame.query_pool, PipelineStage::BottomOfPipe, 1);

        if let Some(render_target) = ctx.display.get_render_target() {
            cmd.transition_image_layout(render_target, ImageLayout::Present);
        }

        cmd.end_label();

        cmd.end();

        ctx.device.wait_transfer();

        cmd.submit2(&ctx.display, frame_id);

        let Ok(_) = ctx.display.present(&ctx.device) else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    pub fn update(&mut self, _ctx: &GfxContext, _delta: f32) {}

    pub fn render<'a>(
        &'a mut self,
        ctx: &mut GfxContext,
        frame: &FrameData,
        render_list: &[RenderObject],
        uniform_cb: &dyn Fn(PassId) -> Option<&'a [u8]>,
    ) -> Result<(), RenderError> {
        for pass in &self.passes {
            tracing::debug!(target: "render", "Begin rendering pass {}", pass.name());
            let uniform_data = uniform_cb(pass.id());

            pass.render(
                ctx,
                frame,
                &self.resource_manager,
                render_list,
                uniform_data,
                self.draw_extent,
            )?;

            tracing::debug!(target: "render", "End rendering pass");
        }

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
