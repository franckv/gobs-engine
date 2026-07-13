use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_render_hal::{RenderHAL, VertexAttribute, create_hal};

use crate::RenderError;

const FRAMES_IN_FLIGHT: usize = 2;

pub struct GfxContext {
    pub hal: Box<dyn RenderHAL>,
    pub color_format: ImageFormat,
    pub depth_format: ImageFormat,
    pub frames_in_flight: usize,
    pub vertex_padding: bool,
    pub world_vertex_attributes: VertexAttribute,
    pub stats_refresh: usize,
}

impl GfxContext {
    pub fn new_frame(&mut self, frame_number: usize) {
        self.hal.new_frame(frame_number);
    }

    pub fn new(name: &str, window: Option<Window>, validation: bool) -> Result<Self, RenderError> {
        let hal = create_hal(name, window, FRAMES_IN_FLIGHT, validation);

        Ok(Self {
            hal,
            color_format: ImageFormat::R16g16b16a16Sfloat,
            depth_format: ImageFormat::D32Sfloat,
            frames_in_flight: FRAMES_IN_FLIGHT,
            vertex_padding: true,
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
