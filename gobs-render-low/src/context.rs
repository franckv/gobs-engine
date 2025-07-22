use std::sync::Arc;

use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{Device, Display, GfxDevice, GfxDisplay, GfxInstance, Instance};

use crate::RenderError;

const FRAMES_IN_FLIGHT: usize = 2;

pub struct GfxContext {
    pub instance: Arc<GfxInstance>,
    pub display: GfxDisplay,
    pub device: Arc<GfxDevice>,
    pub color_format: ImageFormat,
    pub depth_format: ImageFormat,
    pub frames_in_flight: usize,
    pub vertex_padding: bool,
    pub stats_refresh: usize,
}

impl GfxContext {
    pub fn new(name: &str, window: Option<Window>, validation: bool) -> Result<Self, RenderError> {
        let instance = GfxInstance::new(name, window.as_ref(), validation)?;
        let mut display = GfxDisplay::new(instance.clone(), window)?;
        let device = GfxDevice::new(instance.clone(), &display)?;
        display.init(&device, FRAMES_IN_FLIGHT);

        Ok(Self {
            instance,
            display,
            device,
            color_format: ImageFormat::R16g16b16a16Sfloat,
            depth_format: ImageFormat::D32Sfloat,
            frames_in_flight: FRAMES_IN_FLIGHT,
            vertex_padding: true,
            stats_refresh: 60,
        })
    }

    pub fn extent(&self) -> ImageExtent2D {
        self.display.get_extent(&self.device)
    }

    pub fn request_redraw(&self) {
        self.display.request_redraw();
    }
}
