use winit::window::Window;

use gobs_core::ImageExtent2D;
use gobs_render_hal::{RenderHAL, VertexAttribute, create_hal};

use crate::RenderError;

pub struct GfxContext {
    hal: Box<dyn RenderHAL>,
    pub frames_in_flight: usize,
    pub vertex_padding: bool,
    pub world_vertex_attributes: VertexAttribute,
    pub stats_refresh: usize,
}

impl GfxContext {
    pub fn hal(&self) -> &dyn RenderHAL {
        self.hal.as_ref()
    }

    pub fn hal_mut(&mut self) -> &mut dyn RenderHAL {
        self.hal.as_mut()
    }

    pub fn new_frame(&mut self, frame_number: usize) {
        self.hal.new_frame(frame_number);
    }

    pub fn frame_id(&self, frame_number: usize) -> usize {
        self.hal.frame_id(frame_number)
    }

    pub fn new(
        name: &str,
        window: Option<Window>,
        frames_in_flight: usize,
        validation: bool,
    ) -> Result<Self, RenderError> {
        let hal = create_hal(name, window, frames_in_flight, validation);

        Ok(Self {
            hal,
            frames_in_flight,
            vertex_padding: false,
            world_vertex_attributes: VertexAttribute::POSITION
                | VertexAttribute::COLOR
                | VertexAttribute::TEXTURE
                | VertexAttribute::NORMAL
                | VertexAttribute::TANGENT
                | VertexAttribute::BITANGENT,
            stats_refresh: 60,
        })
    }

    pub fn is_minimized(&self) -> bool {
        self.hal.is_minimized()
    }

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
