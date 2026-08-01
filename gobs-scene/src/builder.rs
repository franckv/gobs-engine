use glam::Vec3;

use gobs_core::Color;
use gobs_render::GfxContext;
use gobs_resource::{camera::Camera, light::Light};

use crate::scene::Scene;

pub struct SceneBuilder<'a> {
    gfx: &'a GfxContext,
    camera: Camera,
    camera_position: Vec3,
    light: Light,
    light_position: Vec3,
}

impl<'a> SceneBuilder<'a> {
    pub fn new(gfx: &'a GfxContext) -> Self {
        Self {
            gfx,
            camera: Camera::default(),
            camera_position: Vec3::default(),
            light: Light::default(),
            light_position: Vec3::default(),
        }
    }

    pub fn with_perspective_camera(mut self, yawn: f32, pitch: f32, pos: [f32; 3]) -> Self {
        let extent = self.gfx.extent();

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            60_f32.to_radians(),
            0.1,
            100.,
            yawn.to_radians(),
            pitch.to_radians(),
        );

        self.camera = camera;
        self.camera_position = pos.into();

        self
    }

    pub fn with_ortho_camera(mut self, pos: [f32; 3]) -> Self {
        let extent = self.gfx.extent();

        let camera = Camera::ortho(extent.width as f32, extent.height as f32, 0.1, 100., 0., 0.);

        self.camera = camera;
        self.camera_position = pos.into();

        self
    }

    pub fn with_light(mut self, color: Color, pos: [f32; 3]) -> Self {
        let light = Light::new(color);

        self.light = light;
        self.light_position = pos.into();

        self
    }

    pub fn build(self) -> Scene {
        Scene::new(
            self.gfx,
            self.camera,
            self.camera_position,
            self.light,
            self.light_position,
        )
    }
}
