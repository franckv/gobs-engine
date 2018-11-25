use std::sync::Arc;

use vulkano::swapchain::Surface;
use winit::Window;

pub struct Display {
    surface: Arc<Surface<Window>>
}

impl Display {
    pub fn new(surface: Arc<Surface<Window>>) -> Arc<Display> {
        Arc::new(Display {
            surface: surface
        })
    }

    pub fn surface(&self) -> Arc<Surface<Window>> {
        self.surface.clone()
    }

    pub fn dimensions(&self) -> [u32; 2] {
        let dpi = self.surface.window().get_hidpi_factor();
        let dim: (u32, u32) = self.surface.window().get_inner_size().unwrap().to_physical(dpi).into();
        [dim.0, dim.1]
    }
}
