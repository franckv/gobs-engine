use std::f32::consts::FRAC_PI_2;

use gobs::core::{ImageFormat, Input, Key, MouseButton, Transform, logger};
use gobs::game::GobsContext;
use gobs::graphics::{Camera, ProjectionMode};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct InputManager {
    pub controller: CameraController,
    pub process_updates: bool,
    pub draw_bounds: bool,
    pub draw_ui: bool,
    pub draw_wire: bool,
    pub freeze: bool,
    pub delta: f32,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            controller: CameraController::new(3., 0.4),
            process_updates: true,
            draw_bounds: false,
            draw_ui: false,
            draw_wire: false,
            freeze: false,
            delta: 0.,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.delta = delta;
    }

    pub fn input<Context: GobsContext>(
        &mut self,
        ctx: &mut Context,
        input: Input,
        ui_hovered: bool,
    ) {
        match input {
            Input::KeyPressed(key) => match key {
                Key::P => self.process_updates = !self.process_updates,
                Key::C => self.screenshot(ctx),
                Key::F => {
                    tracing::info!(target: logger::APP, "FPS: {}", if self.delta == 0. {0.} else {1. / self.delta})
                }
                Key::U => self.draw_ui = !self.draw_ui,
                Key::B => self.draw_bounds = !self.draw_bounds,
                Key::Z => self.draw_wire = !self.draw_wire,
                Key::H => ctx.gfx().info(),
                Key::Tab => self.controller.fps_mode = !self.controller.fps_mode,
                _ => {
                    if !ui_hovered {
                        self.controller.key_pressed(key)
                    }
                }
            },
            Input::KeyReleased(key) => self.controller.key_released(key),
            Input::MousePressed(MouseButton::Left) => {
                if !ui_hovered {
                    self.controller.mouse_pressed()
                }
            }
            Input::MouseReleased(MouseButton::Left) => {
                if !ui_hovered {
                    self.controller.mouse_released()
                }
            }
            Input::MouseWheel(delta) => {
                if !ui_hovered {
                    self.controller.mouse_scroll(delta)
                }
            }
            Input::MouseMotion(dx, dy) if !ui_hovered => self.controller.mouse_drag(dx, dy),
            _ => (),
        }
    }

    fn screenshot<Context: GobsContext>(&self, ctx: &mut Context) {
        let label = "draw";
        let filename = "screenshot.png";

        println!("Capture image {}", label);

        let render_target = ctx.get_attachment(label).unwrap();

        let mut data = Vec::new();

        let extent =
            ctx.gfx_mut()
                .capture_image(render_target, &mut data, ImageFormat::R8g8b8a8Unorm);

        let data_len = data.len();

        let image = image::RgbaImage::from_raw(extent.width, extent.height, data).unwrap();

        image.save(filename).unwrap();

        println!("Saved {} ({} bytes)", filename, data_len);
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

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
    reset: bool,
    fps_mode: bool,
}

impl CameraController {
    fn new(speed: f32, sensitivity: f32) -> Self {
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
            reset: false,
            fps_mode: false,
        }
    }

    pub fn lock_mouse(&self) -> bool {
        self.fps_mode
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

        self.reset = false;

        match key {
            Key::W | Key::Up => {
                self.amount_forward = amount;
            }
            Key::S | Key::Down => {
                self.amount_backward = amount;
            }
            Key::A | Key::Left => {
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
            Key::Equals => {
                self.reset = true;
            }
            _ => (),
        }
    }

    pub fn mouse_drag(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if self.fps_mode || self.mouse_pressed {
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
    ) -> bool {
        let forward = camera.dir().normalize();
        let right = camera.right().normalize();
        let up = camera.up().normalize();

        if self.reset {
            camera.yaw = 0.;
            camera.pitch = 0.;
        }

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

        true
    }
}
