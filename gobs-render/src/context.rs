use std::sync::Arc;

use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{Device, Display, GfxDevice, GfxDisplay, GfxInstance, Instance};

pub struct Context {
    pub app_name: String,
    pub instance: Arc<GfxInstance>,
    pub display: GfxDisplay,
    pub device: Arc<GfxDevice>,
    pub color_format: ImageFormat,
    pub depth_format: ImageFormat,
    pub frames_in_flight: usize,
    pub stats_refresh: usize,
    pub frame_number: usize,
    pub vertex_padding: bool,
}

const FRAMES_IN_FLIGHT: usize = 2;

impl Context {
    pub fn new(name: &str, window: Option<Window>, validation: bool) -> Self {
        let instance =
            GfxInstance::new(name, window.as_ref(), validation).expect("Cannot create instance");
        let mut display = GfxDisplay::new(instance.clone(), window).expect("Cannot create display");
        let device = GfxDevice::new(instance.clone(), &display).expect("Cannot create device");
        display.init(&device, FRAMES_IN_FLIGHT);

        Self {
            app_name: name.to_string(),
            instance,
            display,
            device,
            color_format: ImageFormat::R16g16b16a16Sfloat,
            depth_format: ImageFormat::D32Sfloat,
            frames_in_flight: FRAMES_IN_FLIGHT,
            stats_refresh: 60,
            frame_number: 0,
            vertex_padding: true,
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

impl Drop for Context {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop context");
    }
}
