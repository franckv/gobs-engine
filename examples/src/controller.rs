use std::f32::consts::FRAC_PI_2;

use gobs::core::Transform;

use gobs::core::entity::camera::{Camera, ProjectionMode};
use gobs::game::input::Key;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    fov_up: f32,
    fov_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    debug: bool,
    mouse_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.,
            amount_right: 0.,
            amount_forward: 0.,
            amount_backward: 0.,
            amount_up: 0.,
            amount_down: 0.,
            fov_up: 0.,
            fov_down: 0.,
            rotate_horizontal: 0.,
            rotate_vertical: 0.,
            scroll: 0.,
            speed,
            sensitivity,
            debug: false,
            mouse_pressed: false,
        }
    }

    pub fn mouse_pressed(&mut self) {
        self.mouse_pressed = true;
    }

    pub fn mouse_released(&mut self) {
        self.mouse_pressed = false;
    }

    pub fn key_pressed(&mut self, key: Key) {
        self.key_event(key, true);
    }

    pub fn key_released(&mut self, key: Key) {
        self.key_event(key, false);
    }

    fn key_event(&mut self, key: Key, pressed: bool) {
        let amount = if pressed { 2. } else { 0. };

        match key {
            Key::Z | Key::Up => {
                self.amount_forward = amount;
            }
            Key::S | Key::Down => {
                self.amount_backward = amount;
            }
            Key::Q | Key::Left => {
                self.amount_left = amount;
            }
            Key::D | Key::Right => {
                self.amount_right = amount;
            }
            Key::Space => {
                self.amount_up = amount;
            }
            Key::LShift => {
                self.amount_down = amount;
            }
            Key::L => {
                self.debug = true;
            }
            Key::PageUp => {
                self.fov_up = amount;
            }
            Key::PageDown => {
                self.fov_down = amount;
            }
            _ => (),
        }
    }

    pub fn mouse_drag(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if self.mouse_pressed {
            self.rotate_horizontal = -mouse_dx as f32;
            self.rotate_vertical = -mouse_dy as f32;
        }
    }

    pub fn mouse_scroll(&mut self, delta: f32) {
        self.scroll = delta;
    }

    pub fn update_camera(
        &mut self,
        camera: &mut Camera,
        camera_transform: &mut Transform,
        dt: f32,
    ) {
        let forward = camera.dir().normalize();
        let right = camera.right().normalize();
        let up = camera.up().normalize();

        camera_transform.translation +=
            forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera_transform.translation +=
            right * (self.amount_right - self.amount_left) * self.speed * dt;

        camera_transform.translation += forward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.;

        camera_transform.translation += up * (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += self.rotate_vertical * self.sensitivity * dt;

        self.rotate_horizontal = 0.;
        self.rotate_vertical = 0.;

        if let ProjectionMode::Perspective(projection) = &mut camera.mode {
            projection.fovy += (self.fov_up - self.fov_down) * dt;
        }

        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}
