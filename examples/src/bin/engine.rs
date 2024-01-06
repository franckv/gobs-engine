use std::sync::Arc;

use gobs::{
    game::{
        app::{Application, RenderError, Run},
        context::Context,
        input::{Input, Key},
    },
    vulkan::{
        command::{CommandBuffer, CommandPool},
        descriptor::{
            DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBuilder, DescriptorSetPool,
            DescriptorStage, DescriptorType,
        },
        device::Device,
        image::{ColorSpace, Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage},
        pipeline::{
            DynamicStateElem, Pipeline, PipelineLayout, Rect2D, Shader, ShaderType, Viewport,
        },
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
    render_scaling: f32,
    ds_pool: DescriptorSetPool,
    draw_ds_layout: Arc<DescriptorSetLayout>,
    draw_ds: DescriptorSet,
    bg_pipeline: Pipeline,
    bg_pipeline_layout: Arc<PipelineLayout>,
    scene_pipeline: Pipeline,
    scene_pipeline_layout: Arc<PipelineLayout>,
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
            extent,
        );

        let draw_ds_layout = DescriptorSetLayoutBuilder::new()
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone());
        let ds_pool = DescriptorSetPool::new(ctx.device.clone(), draw_ds_layout.clone(), 10);
        let draw_ds = ds_pool.allocate(draw_ds_layout.clone());

        draw_ds
            .update()
            .bind_image(&draw_image, ImageLayout::General)
            .end();

        let compute_shader = Shader::from_file(
            "examples/shaders/sky.comp.spv",
            ctx.device.clone(),
            ShaderType::Compute,
        );

        let bg_pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), Some(draw_ds_layout.clone()));
        let bg_pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(bg_pipeline_layout.clone())
            .compute_shader("main", compute_shader)
            .build();

        let vertex_shader = Shader::from_file(
            "examples/shaders/triangle.vert.spv",
            ctx.device.clone(),
            ShaderType::Vertex,
        );

        let fragment_shader = Shader::from_file(
            "examples/shaders/triangle.frag.spv",
            ctx.device.clone(),
            ShaderType::Fragment,
        );

        let scene_pipeline_layout = PipelineLayout::new(ctx.device.clone(), None);
        let scene_pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(scene_pipeline_layout.clone())
            .vertex_shader("main", vertex_shader)
            .fragment_shader("main", fragment_shader)
            .viewports(vec![Viewport::new(
                0.,
                0.,
                draw_image.extent.width as f32,
                draw_image.extent.height as f32,
            )])
            .scissors(vec![Rect2D::new(
                0,
                0,
                draw_image.extent.width,
                draw_image.extent.height,
            )])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(draw_image.format, None)
            .depth_test_disable()
            .build();

        App {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            draw_image,
            render_scaling: 1.,
            ds_pool,
            draw_ds_layout,
            draw_ds,
            bg_pipeline,
            bg_pipeline_layout,
            scene_pipeline,
            scene_pipeline_layout,
        }
    }

    fn update(&mut self, _ctx: &Context, _delta: f32) {
        log::debug!("Update");
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        log::debug!("Render frame {}", self.frame_number);

        let draw_extent = ImageExtent2D::new(
            (self
                .draw_image
                .extent
                .width
                .min(ctx.surface.get_dimensions().width) as f32
                * self.render_scaling) as u32,
            (self
                .draw_image
                .extent
                .height
                .min(ctx.surface.get_dimensions().height) as f32
                * self.render_scaling) as u32,
        );

        let frame = &self.frames[self.frame_number % FRAMES_IN_FLIGHT];

        frame.render_fence.wait_and_reset();
        assert!(!frame.render_fence.signaled());

        let Ok(image_index) = self.swapchain.acquire_image(&frame.swapchain_semaphore) else {
            return Err(RenderError::Outdated);
        };

        self.draw_image.invalidate();
        self.swapchain_images[image_index as usize].invalidate();

        frame.command_buffer.reset();

        frame.command_buffer.begin();

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::General);

        self.draw_background(&frame.command_buffer, draw_extent);

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::Color);

        self.draw_scene(&frame.command_buffer, draw_extent);

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::TransferSrc);

        let swapchain_image = &mut self.swapchain_images[image_index as usize];

        frame
            .command_buffer
            .transition_image_layout(swapchain_image, ImageLayout::TransferDst);

        frame.command_buffer.copy_image_to_image(
            &self.draw_image,
            draw_extent,
            swapchain_image,
            swapchain_image.extent,
        );

        frame
            .command_buffer
            .transition_image_layout(swapchain_image, ImageLayout::Present);

        frame.command_buffer.end();

        frame.command_buffer.submit2(
            &ctx.queue,
            &frame.swapchain_semaphore,
            &frame.render_semaphore,
            &frame.render_fence,
        );

        let Ok(_) = self
            .swapchain
            .present(image_index, &ctx.queue, &frame.render_semaphore)
        else {
            return Err(RenderError::Outdated);
        };

        self.frame_number += 1;

        log::debug!("End render");

        Ok(())
    }

    fn input(&mut self, _ctx: &Context, input: Input) {
        log::debug!("Input");

        match input {
            Input::KeyPressed(key) => match key {
                Key::E => self.render_scaling = (self.render_scaling + 0.1).min(1.),
                Key::A => self.render_scaling = (self.render_scaling - 0.1).max(0.1),
                _ => (),
            },
            _ => (),
        }
    }

    fn resize(&mut self, ctx: &Context, _width: u32, _height: u32) {
        log::info!("Resize");
        self.resize_swapchain(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    fn clear_background(&self, cmd: &CommandBuffer) {
        let flash = (self.frame_number as f32 / 120.).sin().abs();
        cmd.clear_color(&self.draw_image, [flash, 0., 0., 1.]);
    }

    fn draw_background(&self, cmd: &CommandBuffer, draw_extent: ImageExtent2D) {
        cmd.bind_pipeline(&self.bg_pipeline);
        cmd.bind_descriptor_set(&self.draw_ds, &self.bg_pipeline);
        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);
    }

    fn draw_scene(&self, cmd: &CommandBuffer, draw_extent: ImageExtent2D) {
        cmd.begin_rendering(&self.draw_image, draw_extent, None, false, [1.; 4]);
        cmd.bind_pipeline(&self.scene_pipeline);
        cmd.set_viewport(draw_extent.width, draw_extent.height);
        cmd.draw(3);
        cmd.end_rendering();
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

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
