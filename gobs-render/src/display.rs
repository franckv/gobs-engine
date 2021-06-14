use std::sync::Arc;

use gobs_vulkan as backend;

use super::context::Context;


use backend::command::CommandBuffer;
use backend::framebuffer::Framebuffer;
use backend::image::{ColorSpace, Image, ImageFormat, ImageLayout, ImageUsage};
use backend::physical::PhysicalDevice;
use backend::renderpass::RenderPass;
use backend::surface::{Surface, SurfaceFormat};
use backend::swapchain::{PresentationMode, SwapChain};
use backend::sync::Semaphore;

pub struct Display {
    context: Arc<Context>,
    surface: Arc<Surface>,
    format: SurfaceFormat,
    present: PresentationMode,
    depth_buffer: Image,
    swapchain: SwapChain,
    pub image_count: usize,
    renderpass: Arc<RenderPass>,
    framebuffers: Vec<Framebuffer>,
    width: u32,
    height: u32
}

impl Display {
    pub fn new(context: Arc<Context>,
               surface: Arc<Surface>,
               format: SurfaceFormat,
               renderpass: Arc<RenderPass>) -> Self {

        let (width, height) = surface.get_extent(context.device_ref());

        let depth_buffer =
            Self::create_depth_buffer(&context, width, height);

        let present = Self::get_presentation_mode(&context, &surface);

        let image_count = Self::get_image_count(&context, &surface);

        let swapchain = SwapChain::new(
            context.device(),
            surface.clone(),
            format,
            present,
            image_count,
            None);

        let images = swapchain.create_images();

        let mut framebuffers = Vec::new();

        for image in images {
            framebuffers.push(
                Framebuffer::new(context.device(),
                                 image, &depth_buffer, renderpass.clone())
            )
        }

        Display {
            context,
            surface,
            format,
            present,
            depth_buffer,
            swapchain,
            image_count,
            renderpass,
            framebuffers,
            width,
            height
        }
    }

    pub fn get_surface_format(surface: &Arc<Surface>,
                          p_device: &PhysicalDevice) -> SurfaceFormat {
        let formats =
            surface.get_available_format(p_device);

        *formats.iter().find(|f| {
            f.format == ImageFormat::B8g8r8a8Unorm &&
                f.color_space == ColorSpace::SrgbNonlinear
        }).unwrap()
    }

    pub fn context_ref(&self) -> &Arc<Context> {
        &self.context
    }

    pub fn surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    pub fn surface_ref(&self) -> &Arc<Surface> {
        &self.surface
    }

    pub fn format(&self) -> SurfaceFormat {
        self.format
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn framebuffer(&self, idx: usize) -> &Framebuffer {
        &self.framebuffers[idx]
    }

    pub fn renderpass(&self) -> Arc<RenderPass> {
        self.renderpass.clone()
    }

    fn resize(&mut self) {
        debug!("Resize");
        
        self.context.device_ref().wait();

        let dim = self.surface.get_extent(self.context.device_ref());

        self.width = dim.0;
        self.height = dim.1;

        self.depth_buffer =
            Self::create_depth_buffer(&self.context, self.width, self.height);

        self.swapchain = SwapChain::new(
            self.context.device(),
            self.surface.clone(),
            self.format,
            self.present,
            self.image_count,
            Some(&self.swapchain));

        let images = self.swapchain.create_images();
        let mut framebuffers = Vec::new();

        for image in images {
            framebuffers.push(
                Framebuffer::new(self.context.device(),
                                 image, &self.depth_buffer,
                                 self.renderpass.clone())
            )
        }

        self.framebuffers = framebuffers;
    }

    fn get_presentation_mode(context: &Context,
                             surface: &Arc<Surface>) -> PresentationMode {
        let presents =
            surface.get_available_presentation_modes(context.device_ref());

        *presents.iter().find(|&&p| {
            p == PresentationMode::Fifo
        }).unwrap()
    }

    fn get_image_count(context: &Context,
                       surface: &Arc<Surface>) -> usize {
        let caps =
            surface.get_capabilities(context.device_ref());

        let mut count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && count > caps.max_image_count {
            count = caps.max_image_count;
        }

        count
    }

    pub fn next_image(&mut self, signal: &Semaphore) -> Result<usize, ()> {
        match self.swapchain.acquire_image(signal) {
            Ok(index) => {
                Ok(index)
            },
            Err(_) => {
                self.resize();
                Err(())
            }
        }
    }

    pub fn present(&mut self, idx: usize, wait: &Semaphore) -> Result<(), ()> {
        match self.swapchain.present(idx, self.context.queue(), wait) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.resize();
                Err(())
            }
        }
    }

    fn create_depth_buffer(context: &Context, width: u32, height: u32) -> Image {
        let depth_buffer = Image::new(context.device(),
                                      ImageFormat::D32Sfloat,
                                      ImageUsage::Depth,
                                      width, height);

        let mut command_buffer = CommandBuffer::new(
            context.device(),
            context.command_pool());

        command_buffer.begin();
        command_buffer.transition_image_layout(&depth_buffer,
                                               ImageLayout::Undefined,
                                               ImageLayout::Depth);
        command_buffer.end();

        command_buffer.submit_now(context.queue(), None);

        depth_buffer
    }
}
