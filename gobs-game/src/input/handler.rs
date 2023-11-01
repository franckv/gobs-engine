use log::*;
use winit;
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};

use crate::input::Input;

#[derive(Debug, PartialEq)]
pub enum Event {
    Resize(u32, u32),
    Input(Input),
    Redraw,
    Cleared,
    Close,
    Continue,
}

impl Event {
    pub fn new(event: winit::event::Event<'_, ()>) -> Self {
        let mut status = Event::Continue;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => status = Event::Close,
                WindowEvent::Resized(physical_size) => {
                    status = Event::Resize(physical_size.width, physical_size.height)
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    status = Event::Resize(new_inner_size.width, new_inner_size.height)
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key_code),
                            state,
                            ..
                        },
                    ..
                } => match key_code {
                    VirtualKeyCode::Escape => status = Event::Close,
                    _ => {
                        let key = key_code.into();
                        match state {
                            ElementState::Pressed => status = Event::Input(Input::KeyPressed(key)),
                            ElementState::Released => {
                                status = Event::Input(Input::KeyReleased(key))
                            }
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    status = Event::Input(Input::CursorMoved(position.x, position.y));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                            scroll as f32
                        }
                    };
                    status = Event::Input(Input::MouseWheel(delta));
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => {
                    status = match state {
                        ElementState::Pressed => Event::Input(Input::MousePressed),
                        ElementState::Released => Event::Input(Input::MouseReleased),
                    }
                }
                _ => (),
            },
            winit::event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => status = Event::Input(Input::MouseMotion(delta.0, delta.1)),
            winit::event::Event::RedrawRequested(_) => status = Event::Redraw,
            winit::event::Event::MainEventsCleared {} => status = Event::Cleared,
            _ => (),
        }

        debug!("Status={:?}", status);
        status
    }
}
