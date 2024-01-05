use std::sync::Arc;

use gobs::{
    game::{
        app::{Application, RenderError, Run},
        context::Context,
        input::Input,
    },
    vulkan::{
        command::{CommandBuffer, CommandPool},
        descriptor::{
            DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBuilder, DescriptorSetPool,
            DescriptorStage, DescriptorType,
        },
        device::Device,
        image::{ColorSpace, Image, ImageFormat, ImageLayout, ImageUsage},
        pipeline::{Pipeline, PipelineLayout, Shader, ShaderType},
        queue::QueueFamily,
        swapchain::{PresentationMode, SwapChain},
        sync::{Fence, Semaphore},
    },
};

const FRAMES_IN_FLIGHT: usize = 2;

struct FrameData {
    pub command_buffer: CommandBuffer,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
    pub render_fence: Fence,
}

impl FrameData {
    pub fn new(device: Arc<Device>, queue_family: &QueueFamily) -> Self {
        let command_pool = CommandPool::new(device.clone(), queue_family);
        let command_buffer = CommandBuffer::new(device.clone(), command_pool);

        let swapchain_semaphore = Semaphore::new(device.clone());
        let render_semaphore = Semaphore::new(device.clone());
        let render_fence = Fence::new(device, true);

        FrameData {
            command_buffer,
            swapchain_semaphore,
            render_semaphore,
            render_fence,
        }
    }
}

struct App {
    frame_number: usize,
    frames: [FrameData; FRAMES_IN_FLIGHT],
    swapchain: SwapChain,
    swapchain_images: Vec<Image>,
    draw_image: Image,
    ds_pool: DescriptorSetPool,
    ds_layout: Arc<DescriptorSetLayout>,
    ds: DescriptorSet,
    pipeline: Pipeline,
    pipeline_layout: Arc<PipelineLayout>,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        log::info!("Create");

        let frames = [
            FrameData::new(ctx.device.clone(), ctx.queue.family()),
            FrameData::new(ctx.device.clone(), ctx.queue.family()),
        ];

        let swapchain = Self::create_swapchain(ctx);
        let swapchain_images = swapchain.create_images();

        let extent = ctx.surface.get_extent(ctx.device.clone());
        let draw_image = Image::new(
            ctx.device.clone(),
            ImageFormat::R16g16b16a16Sfloat,
            ImageUsage::Color,
            extent.0,
            extent.1,
        );

        let ds_layout = DescriptorSetLayoutBuilder::new()
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone());
        let ds_pool = DescriptorSetPool::new(ctx.device.clone(), ds_layout.clone(), 10);
        let ds = ds_pool.allocate(ds_layout.clone());

        ds.update()
            .bind_image(&draw_image, ImageLayout::General)
            .end();

        let shader = Shader::from_file(
            "examples/shaders/sky.comp.spv",
            ctx.device.clone(),
            ShaderType::Compute,
        );

        let pipeline_layout = PipelineLayout::new(ctx.device.clone(), ds_layout.clone());
        let pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .compute_shader("main", shader)
            .build();

        App {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            draw_image,
            ds_pool,
            ds_layout,
            ds,
            pipeline,
            pipeline_layout,
        }
    }

    fn update(&mut self, _ctx: &Context, _delta: f32) {
        log::debug!("Update");
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        log::debug!("Render");
        log::trace!("Frame {}", self.frame_number);

        let frame = &mut self.frames[self.frame_number % FRAMES_IN_FLIGHT];

        frame.render_fence.wait_and_reset();
        assert!(!frame.render_fence.signaled());

        let image_index = self
            .swapchain
            .acquire_image(&frame.swapchain_semaphore)
            .expect("Failed to acquire image");
        let swapchain_image = &self.swapchain_images[image_index as usize];

        frame.command_buffer.reset();

        frame.command_buffer.begin();

        frame.command_buffer.transition_image_layout(
            &self.draw_image,
            ImageLayout::Undefined,
            ImageLayout::General,
        );

        Self::draw_background(
            &mut frame.command_buffer,
            &self.pipeline,
            &self.ds,
            self.draw_image.width,
            self.draw_image.height,
        );

        frame.command_buffer.transition_image_layout(
            &self.draw_image,
            ImageLayout::General,
            ImageLayout::TransferSrc,
        );
        frame.command_buffer.transition_image_layout(
            swapchain_image,
            ImageLayout::Undefined,
            ImageLayout::TransferDst,
        );

        frame
            .command_buffer
            .copy_image_to_image(&self.draw_image, swapchain_image);

        frame.command_buffer.transition_image_layout(
            swapchain_image,
            ImageLayout::TransferDst,
            ImageLayout::Present,
        );

        frame.command_buffer.end();

        frame.command_buffer.submit2(
            &ctx.queue,
            &frame.swapchain_semaphore,
            &frame.render_semaphore,
            &frame.render_fence,
        );

        self.swapchain
            .present(image_index, &ctx.queue, &frame.render_semaphore)
            .expect("Failed to present image");

        self.frame_number += 1;

        log::debug!("End render");

        Ok(())
    }

    fn input(&mut self, _ctx: &Context, _input: Input) {
        log::debug!("Input");
    }

    fn resize(&mut self, _ctx: &Context, _width: u32, _height: u32) {
        log::info!("Resize");
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    fn clear_background(cmd: &mut CommandBuffer, frame_number: usize, image: &Image) {
        let flash = (frame_number as f32 / 120.).sin().abs();
        cmd.clear_color(image, [flash, 0., 0., 1.]);
    }

    fn draw_background(
        cmd: &mut CommandBuffer,
        pipeline: &Pipeline,
        ds: &DescriptorSet,
        width: u32,
        height: u32,
    ) {
        cmd.bind_pipeline(pipeline);
        cmd.bind_descriptor_set(ds, &pipeline);
        cmd.dispatch(width / 16 + 1, height / 16 + 1, 1);
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
}
fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new().run::<App>();
}
