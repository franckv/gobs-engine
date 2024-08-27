use anyhow::Result;
use std::sync::Arc;
use winit::window::Window;

use crate::Renderer;

pub trait Instance<R: Renderer> {
    fn new(name: &str, window: Option<&Window>, validation: bool) -> Result<Arc<Self>>;
}
