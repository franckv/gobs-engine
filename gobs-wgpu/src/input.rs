use winit::event::*;

use crate::scene::Scene;

pub struct Input {
    pub mouse_pressed: bool,
}

impl Input {
    pub fn mouse_pressed(&self) -> bool {
        self.mouse_pressed
    }

    pub fn new() -> Self {
        Input {
            mouse_pressed: false,
        }
    }

    pub fn input(&mut self, scene: &mut Scene, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => scene.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                scene.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn mouse_input(&mut self, scene: &mut Scene, mouse_dx: f64, mouse_dy: f64) {
        scene.camera_controller.process_mouse(mouse_dx, mouse_dy)
    }
}
