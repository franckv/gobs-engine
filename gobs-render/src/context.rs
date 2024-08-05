use std::sync::Arc;

use winit::window::Window;

use gobs_gfx as gfx;

use gfx::{
    Device, Display, GfxDevice, GfxDisplay, GfxInstance, ImageExtent2D, ImageFormat, Instance,
};

pub struct Context {
    pub app_name: String,
    pub instance: Arc<GfxInstance>,
    pub display: Arc<GfxDisplay>,
    pub device: Arc<GfxDevice>,
    pub color_format: ImageFormat,
    pub depth_format: ImageFormat,
    pub frames_in_flight: usize,
    pub stats_refresh: usize,
    pub frame_number: usize,
}

impl Context {
    pub fn new(name: &str, window: Option<Window>) -> Self {
        let instance =
            GfxInstance::new(name, window.as_ref(), true).expect("Cannot create instance");
        let display = GfxDisplay::new(instance.clone(), window).expect("Cannot create display");
        let device =
            GfxDevice::new(instance.clone(), display.clone()).expect("Cannot create device");

        Context {
            app_name: name.to_string(),
            instance,
            display,
            device,
            color_format: ImageFormat::R16g16b16a16Sfloat,
            depth_format: ImageFormat::D32Sfloat,
            frames_in_flight: 2,
            stats_refresh: 60,
            frame_number: 0,
        }
    }

    pub fn extent(&self) -> ImageExtent2D {
        //TODO: self.surface.get_extent(self.device.clone())
        self.display.get_extent()
    }

    pub fn frame_id(&self) -> usize {
        self.frame_number % self.frames_in_flight
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        log::debug!("Drop context");
    }
}
