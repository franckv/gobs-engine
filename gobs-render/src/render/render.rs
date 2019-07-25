use std::mem;
use std::sync::Arc;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::swapchain;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain};
use vulkano::sync::{GpuFuture, now, FlushError};

use winit::Window;

use context::Context;
use display::Display;
use render::{Batch, Command};
use scene::Color;

pub struct Renderer {
    context: Arc<Context>,
    display: Arc<Display>,
    swapchain: Arc<Swapchain<Window>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    last_frame: Box<dyn GpuFuture + Sync + Send>
}

impl Renderer {
    pub fn new(context: Arc<Context>, display: Arc<Display>) -> Renderer {
        let (swapchain, images) = {
            let caps = display.surface().capabilities(
                context.device().physical_device()).expect("error");
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;

            Swapchain::new(context.device(), display.surface(), caps.min_image_count,
                format, display.dimensions(), 1, caps.supported_usage_flags, &context.queue(),
                SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None).expect("error")
        };

        let render_pass = Self::create_render_pass(context.device(), swapchain.format());

        let framebuffers = Self::create_framebuffer(render_pass.clone(), context.clone(),
            swapchain.dimensions(), images);

        Renderer {
            context: context.clone(),
            display: display,
            swapchain: swapchain,
            framebuffers: framebuffers,
            render_pass: render_pass,
            last_frame: Box::new(now(context.device()))
                as Box<dyn GpuFuture + Sync + Send>
        }
    }

    pub fn framebuffer(&self, id: usize) -> Arc<dyn FramebufferAbstract + Send + Sync> {
        self.framebuffers[id].clone()
    }

    pub fn new_frame(&mut self) -> Result<(usize, Box<dyn GpuFuture + Send + Sync>), AcquireError> {
        let (idx, acquire_future) = match swapchain::acquire_next_image(
            self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                debug!("OutOfDate");
                self.recreate_swapchain();
                return Err(AcquireError::OutOfDate)
            },
            Err(err) => panic!("{:?}", err)
        };

        Ok((idx, Box::new(acquire_future)))
    }

    pub fn submit(&mut self, command: Command) {
        self.submit_list(vec!(command));
    }

    pub fn submit_list(&mut self, command_list: Vec<Command>) {
        self.last_frame.cleanup_finished();

        if let Ok((id, future)) = self.new_frame() {
            let last_frame = mem::replace(&mut self.last_frame,
                Box::new(now(self.context.device())) as Box<dyn GpuFuture + Sync + Send>);

            self.last_frame = Box::new(last_frame.join(future));

            let clear_color: [f32; 4] = Color::blue().into();

            let mut primary = AutoCommandBufferBuilder::primary_one_time_submit(
                self.context.device(), self.context.queue().family()).unwrap()
                .begin_render_pass(self.framebuffer(id),
                    true, vec![clear_color.into(), 1f32.into()]).unwrap();


            for command in command_list {
                let command = command.command();

                unsafe {
                    primary = primary.execute_commands(command).unwrap();
                }
            }

            let last_frame = mem::replace(&mut self.last_frame,
                Box::new(now(self.context.device())) as Box<dyn GpuFuture + Sync + Send>);

            let command_buffer = primary.end_render_pass().unwrap()
                .build().unwrap();

            let future = last_frame
                .then_execute(self.context.queue(), command_buffer).unwrap()
                .then_swapchain_present(self.context.queue(),
                    self.swapchain.clone(), id)
                .then_signal_fence_and_flush();

            self.last_frame = match future {
                Ok(future) => {
                    Box::new(future) as Box<_>
                },
                Err(FlushError::OutOfDate) => {
                    Box::new(now(self.context.device())) as Box<_>
                },
                Err(e) => {
                    warn!("{:?}", e);
                    Box::new(now(self.context.device())) as Box<_>
                }
            }
        }
    }

    pub fn create_batch(&self) -> Batch {
        Batch::new(self.display.clone(), self.context.clone(), self.render_pass.clone())
    }

    fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        Arc::new(single_pass_renderpass!(device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: Store,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            }, pass: {
                color: [color],
                depth_stencil: {depth}
                resolve: [],
            }
        ).unwrap())
    }

    fn create_framebuffer(render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        context: Arc<Context>, dimensions: [u32; 2],
        images: Vec<Arc<SwapchainImage<Window>>>)
        -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {

        let depth_buffer = AttachmentImage::transient(context.device(),
            dimensions.clone(), Format::D16Unorm).unwrap();

        images.iter().map(|image| {
            Arc::new(Framebuffer::start(render_pass.clone())
                     .add(image.clone()).unwrap()
                     .add(depth_buffer.clone()).unwrap()
                     .build().unwrap()) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>()
    }

    fn recreate_swapchain(&mut self) {
        let (swapchain, images) = self.swapchain.recreate_with_dimension(
            self.display.dimensions()).unwrap();

        mem::replace(&mut self.swapchain, swapchain);

        self.framebuffers = Self::create_framebuffer(self.render_pass.clone(),
            self.context.clone(), self.swapchain.dimensions(), images)
    }
}
