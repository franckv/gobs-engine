use std::sync::Arc;

use anyhow::Result;
use winit::window::Window;

use gobs_core::ImageExtent2D;

pub trait Display {
    type GfxDisplay;
    type GfxDevice;
    type GfxImage;
    type GfxInstance;

    fn new(instance: Arc<Self::GfxInstance>, window: Option<Window>) -> Result<Self>
    where
        Self: Sized;
    fn init(&mut self, device: &Self::GfxDevice, frames_in_flight: usize);
    fn get_extent(&self, device: &Self::GfxDevice) -> ImageExtent2D;
    fn get_render_target(&mut self) -> &mut Self::GfxImage;
    fn acquire(&mut self, frame: usize) -> Result<()>;
    fn present(&mut self, device: &Self::GfxDevice, frame: usize) -> Result<()>;
    fn resize(&mut self, device: &Self::GfxDevice);
    fn request_redraw(&self);
    fn is_minimized(&self) -> bool;
}
