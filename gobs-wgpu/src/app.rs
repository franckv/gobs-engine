use log::*;
use winit::event::*;
use winit::event_loop::*;
use winit::window::{Window, WindowBuilder};

use crate::Gfx;
use crate::Input;
use crate::scene::Scene;

use gobs_utils::timer::Timer;

pub struct Application {
    event_loop: EventLoop<()>,
    window: Window,
    gfx: Gfx,
    scene: Scene,
    input: Input
}

impl Application {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let gfx = Gfx::new(&window).await;
        let scene = Scene::new(&gfx).await;
        let input = Input::new();

        Application {
            event_loop,
            window,
            gfx,
            scene,
            input
        }
    }

    pub fn run(mut self) {
        let mut timer = Timer::new();

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion{ delta },
                    ..
                } => if self.input.mouse_pressed() {
                    self.input.mouse_input(&mut self.scene, delta.0, delta.1)
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => if !self.input.input(&mut self.scene, event) {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.gfx.resize(physical_size.width, physical_size.height);
                            self.scene.resize(&self.gfx, physical_size.width, physical_size.height);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.gfx.resize(new_inner_size.width, new_inner_size.height);
                            self.scene.resize(&self.gfx, new_inner_size.width, new_inner_size.height);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    self.scene.update(&self.gfx, timer.delta());
                    match self.gfx.render(&self.scene) {
                        Ok(_) => {},
                        Err(wgpu::SurfaceError::Lost) => {
                            self.gfx.resize(self.gfx.width(), self.gfx.height());
                            self.scene.resize(&self.gfx, self.gfx.width(), self.gfx.height());
                        },
                        Err(e) => error!("{:?}", e)
                    }
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}