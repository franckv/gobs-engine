use log::*;
use winit::event::*;
use winit::event_loop::*;
use winit::window::{Window, WindowBuilder};

use crate::State;

use gobs_utils::timer::Timer;

pub struct Application {
    event_loop: EventLoop<()>,
    window: Window,
    state: State
}

impl Application {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let state = State::new(&window).await;

        Application {
            event_loop,
            window,
            state
        }
    }

    pub fn run(mut self) {
        let mut timer = Timer::new();

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion{ delta },
                    ..
                } => if self.state.mouse_pressed {
                    self.state.mouse_input(delta.0, delta.1)
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => if !self.state.input(event) {
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
                            self.state.resize(physical_size.width, physical_size.height);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.state.resize(new_inner_size.width, new_inner_size.height);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    self.state.update(timer.delta());
                    match self.state.render() {
                        Ok(_) => {},
                        Err(wgpu::SurfaceError::Lost) => {
                            self.state.redraw()
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