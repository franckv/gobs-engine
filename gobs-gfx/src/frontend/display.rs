use std::sync::Arc;

use anyhow::Result;
use winit::window::Window;

use crate::{GfxDevice, GfxDisplay, GfxImage, GfxInstance, ImageExtent2D};

pub enum DisplayType {
    NullDisplay,
    VideoDisplay(GfxDisplay),
}

pub trait Display {
    fn new(instance: Arc<GfxInstance>, window: Option<Window>) -> Result<DisplayType>;
    fn init(&mut self, device: &GfxDevice, frames_in_flight: usize);
    fn get_extent(&self, device: &GfxDevice) -> ImageExtent2D;
    fn get_render_target(&mut self) -> &mut GfxImage;
    fn acquire(&mut self, frame: usize) -> Result<()>;
    fn present(&mut self, device: &GfxDevice, frame: usize) -> Result<()>;
    fn resize(&mut self, device: &GfxDevice);
}

impl DisplayType {
    pub fn init(&mut self, device: &GfxDevice, frames_in_flight: usize) {
        match self {
            DisplayType::NullDisplay => {}
            DisplayType::VideoDisplay(display) => display.init(device, frames_in_flight),
        }
    }

    pub fn get_extent(&self, device: &GfxDevice) -> ImageExtent2D {
        match self {
            DisplayType::NullDisplay => ImageExtent2D::new(0, 0),
            DisplayType::VideoDisplay(display) => display.get_extent(device),
        }
    }

    pub fn get_render_target(&mut self) -> &mut GfxImage {
        match self {
            DisplayType::NullDisplay => unimplemented!(),
            DisplayType::VideoDisplay(display) => display.get_render_target(),
        }
    }

    pub fn acquire(&mut self, frame: usize) -> Result<()> {
        match self {
            DisplayType::NullDisplay => Ok(()),
            DisplayType::VideoDisplay(display) => display.acquire(frame),
        }
    }

    pub fn present(&mut self, device: &GfxDevice, frame: usize) -> Result<()> {
        match self {
            DisplayType::NullDisplay => Ok(()),
            DisplayType::VideoDisplay(display) => display.present(device, frame),
        }
    }

    pub fn resize(&mut self, device: &GfxDevice) {
        match self {
            DisplayType::NullDisplay => (),
            DisplayType::VideoDisplay(display) => display.resize(device),
        }
    }

    pub fn request_redraw(&self) {
        match self {
            DisplayType::NullDisplay => (),
            DisplayType::VideoDisplay(display) => {
                display.surface.window.request_redraw();
            }
        }
    }
}
