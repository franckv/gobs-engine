use std::sync::Arc;
use winit::window::Window;

use crate::{GfxError, Renderer};

pub trait Instance<R: Renderer> {
    fn new(name: &str, window: Option<&Window>, validation: bool) -> Result<Arc<Self>, GfxError>;
}
