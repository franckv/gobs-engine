use std::sync::Arc;

use anyhow::Result;
use winit::window::Window;

pub trait Instance {
    fn new(name: &str, window: Option<&Window>, validation: bool) -> Result<Arc<Self>>;
}
