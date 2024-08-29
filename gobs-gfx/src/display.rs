use std::sync::Arc;

use anyhow::Result;
use winit::window::Window;

use gobs_core::ImageExtent2D;

use crate::Renderer;

pub trait Display<R: Renderer> {
    fn new(instance: Arc<R::Instance>, window: Option<Window>) -> Result<Self>
    where
        Self: Sized;
    fn init(&mut self, device: &R::Device, frames_in_flight: usize);
    fn get_extent(&self, device: &R::Device) -> ImageExtent2D;
    fn get_render_target(&mut self) -> Option<&mut R::Image>;
    fn acquire(&mut self, frame: usize) -> Result<()>;
    fn present(&mut self, device: &R::Device, frame: usize) -> Result<()>;
    fn resize(&mut self, device: &R::Device);
    fn request_redraw(&self);
    fn is_minimized(&self) -> bool;
}
