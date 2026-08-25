use winit::window::Window;

use gobs_core::{GobsConfig, ImageExtent2D};
use gobs_render_hal::{RenderHAL, create_hal};

pub struct GfxContext {
    hal: Box<dyn RenderHAL>,
}

impl GfxContext {
    pub fn hal(&self) -> &dyn RenderHAL {
        self.hal.as_ref()
    }

    pub fn hal_mut(&mut self) -> &mut dyn RenderHAL {
        self.hal.as_mut()
    }

    pub fn frames_in_flight(&self) -> usize {
        self.hal.frames_in_flight()
    }

    pub fn new_frame(&mut self, frame_number: usize) {
        self.hal.new_frame(frame_number);
    }

    pub fn frame_id(&self, frame_number: usize) -> usize {
        self.hal.frame_id(frame_number)
    }

    pub fn new(name: &str, window: Option<Window>, config: GobsConfig, validation: bool) -> Self {
        let hal = create_hal(name, window, config, validation);

        Self { hal }
    }

    pub fn is_minimized(&self) -> bool {
        self.hal.is_minimized()
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn resize(&mut self) {
        self.hal.resize();
    }

    pub fn extent(&self) -> ImageExtent2D {
        self.hal.get_extent()
    }

    pub fn request_redraw(&mut self) {
        self.hal.request_redraw();
    }
}
