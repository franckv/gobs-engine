use std::f32::consts::FRAC_PI_2;

use gobs::core::Transform;
use gobs::game::input::{Input, Key, MouseButton};
use gobs::resource::entity::camera::{Camera, ProjectionMode};

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

    pub fn input(&mut self, input: Input, ui_hovered: bool) {
        match input {
            Input::KeyPressed(key) => self.key_pressed(key),
            Input::KeyReleased(key) => self.key_released(key),
            Input::MousePressed(MouseButton::Left) => {
                if !ui_hovered {
                    self.mouse_pressed()
                }
            }
            Input::MouseReleased(MouseButton::Left) => {
                if !ui_hovered {
                    self.mouse_released()
                }
            }
            Input::MouseWheel(delta) => {
                if !ui_hovered {
                    self.mouse_scroll(delta)
                }
            }
            Input::MouseMotion(dx, dy) => {
                if !ui_hovered {
                    self.mouse_drag(dx, dy)
                }
            }
            _ => (),
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

        camera_transform
            .translate(forward * (self.amount_forward - self.amount_backward) * self.speed * dt);
        camera_transform
            .translate(right * (self.amount_right - self.amount_left) * self.speed * dt);
        camera_transform.translate(forward * self.scroll * self.speed * self.sensitivity * dt);
        camera_transform.translate(up * (self.amount_up - self.amount_down) * self.speed * dt);

        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += self.rotate_vertical * self.sensitivity * dt;

        self.scroll = 0.;
        self.rotate_horizontal = 0.;
        self.rotate_vertical = 0.;

        if let ProjectionMode::Perspective(projection) = &mut camera.mode {
            projection.fovy += (self.fov_up - self.fov_down) * dt;
        }

        camera.pitch = camera.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
    }
}
