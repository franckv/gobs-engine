use log::*;
use winit::event::*;
use winit::window::Window;

use crate::Gfx;
use crate::scene::Scene;

pub struct State {
    gfx: Gfx,
    scene: Scene,
    pub mouse_pressed: bool
}

impl State {
    pub async fn new(window: Window) -> Self {
        info!("init state");

        let gfx = Gfx::new(window).await;
        let scene = Scene::new(&gfx).await;


        Self {
            gfx,
            scene,
            mouse_pressed: false
        }
    }

    pub fn window(&self) -> &Window {
        &self.gfx.window()
    }

    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        &self.gfx.size()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        info!("resize");

        if new_size.width > 0 && new_size.height > 0 {
            self.gfx.resize(new_size);
            self.scene.resize(&self.gfx, new_size);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.scene.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.scene.camera_controller.process_scroll(delta);
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
            _ => false
        }
    }

    pub fn mouse_input(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.scene.camera_controller.process_mouse(mouse_dx, mouse_dy)
    }

    pub fn update(&mut self, dt: f32) {
        self.scene.update(dt);

        self.gfx.queue().write_buffer(&self.scene.camera().resource.buffer, 0, bytemuck::cast_slice(&[self.scene.camera().resource.uniform]));
        self.gfx.queue().write_buffer(&self.scene.light().resource.buffer, 0, bytemuck::cast_slice(&[self.scene.light().resource.uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.gfx.render(&self.scene)
    }
}