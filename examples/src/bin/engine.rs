use std::sync::Arc;

use gobs::{
    game::{
        app::{Application, RenderError, Run},
        context::Context,
        input::Input,
    },
    vulkan::{
        command::{CommandBuffer, CommandPool},
        device::Device,
        image::{ColorSpace, Image, ImageFormat, ImageLayout},
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
}

impl Run for App {
    async fn create(context: &Context) -> Self {
        log::info!("Create");

        let frames = [
            FrameData::new(context.device.clone(), context.queue.family()),
            FrameData::new(context.device.clone(), context.queue.family()),
        ];

        let swapchain = Self::create_swapchain(context);
        let swapchain_images = swapchain.create_images();

        App {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
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
        let image = &self.swapchain_images[image_index as usize];

        frame.command_buffer.reset();

        frame.command_buffer.begin();

        frame.command_buffer.transition_image_layout(
            image,
            ImageLayout::Undefined,
            ImageLayout::General,
        );

        let flash = (self.frame_number as f32 / 120.).sin().abs();
        frame.command_buffer.clear_color(image, [flash, 0., 0., 1.]);

        frame.command_buffer.transition_image_layout(
            image,
            ImageLayout::General,
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
