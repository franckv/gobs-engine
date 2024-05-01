use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::keyboard::NamedKey;
use winit::{self, keyboard};

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
    pub fn new(event: winit::event::Event<()>) -> Self {
        let mut status = Event::Continue;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => status = Event::Close,
                WindowEvent::Resized(physical_size) => {
                    status = Event::Resize(physical_size.width, physical_size.height)
                }
                /*WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    status = Event::Resize(new_inner_size.width, new_inner_size.height)
                }*/
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: key_code,
                            state,
                            ..
                        },
                    ..
                } => match key_code {
                    keyboard::Key::Named(NamedKey::Escape) => status = Event::Close,
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
                WindowEvent::RedrawRequested => status = Event::Redraw,
                _ => (),
            },
            winit::event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => status = Event::Input(Input::MouseMotion(delta.0, delta.1)),

            winit::event::Event::AboutToWait {} => status = Event::Cleared,
            _ => (),
        }

        log::trace!("Status={:?}", status);
        status
    }
}
