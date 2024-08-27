use std::{marker::PhantomData, sync::Arc};

use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{Device, Display, Instance, Renderer};

pub struct Context<R: Renderer> {
    pub app_name: String,
    pub renderer: PhantomData<R>,
    pub instance: Arc<R::Instance>,
    pub display: R::Display,
    pub device: Arc<R::Device>,
    pub color_format: ImageFormat,
    pub depth_format: ImageFormat,
    pub frames_in_flight: usize,
    pub stats_refresh: usize,
    pub frame_number: usize,
}

const FRAMES_IN_FLIGHT: usize = 2;

impl<R: Renderer> Context<R> {
    pub fn new(name: &str, window: Option<Window>) -> Self {
        let instance =
            R::Instance::new(name, window.as_ref(), true).expect("Cannot create instance");
        let mut display = R::Display::new(instance.clone(), window).expect("Cannot create display");
        let device = R::Device::new(instance.clone(), &display).expect("Cannot create device");
        display.init(&device, FRAMES_IN_FLIGHT);

        Self {
            app_name: name.to_string(),
            renderer: Default::default(),
            instance,
            display,
            device,
            color_format: ImageFormat::R16g16b16a16Sfloat,
            depth_format: ImageFormat::D32Sfloat,
            frames_in_flight: FRAMES_IN_FLIGHT,
            stats_refresh: 60,
            frame_number: 0,
        }
    }

    pub fn extent(&self) -> ImageExtent2D {
        self.display.get_extent(&self.device)
    }

    pub fn frame_id(&self) -> usize {
        self.frame_number % self.frames_in_flight
    }

    pub fn request_redraw(&self) {
        self.display.request_redraw();
    }
}

impl<R: Renderer> Drop for Context<R> {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop context");
    }
}
