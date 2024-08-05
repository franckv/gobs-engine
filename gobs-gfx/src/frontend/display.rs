use std::sync::Arc;

use anyhow::Result;
use winit::window::Window;

use crate::{GfxDevice, GfxInstance, ImageExtent2D};

pub trait Display {
    fn new(instance: Arc<GfxInstance>, window: Option<Window>) -> Result<Arc<Self>>;
    fn init(&mut self, device: &GfxDevice);
    fn get_extent(&self) -> ImageExtent2D;
    fn acquire(&mut self) -> Result<()>;
}
