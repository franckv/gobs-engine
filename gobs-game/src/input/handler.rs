use log::*;
use winit;
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};

use crate::input::Key;

#[derive(Debug, PartialEq)]
pub enum Event {
    Resize(u32, u32),
    KeyPressed(Key),
    KeyReleased(Key),
    MouseWheel(f32),
    MouseMotion(f64, f64),
    MousePressed,
    MouseReleased,
    Redraw,
    Cleared,
    Close,
    Continue,
}

pub struct InputHandler {}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {}
    }

    pub fn read_inputs(&mut self, event: winit::event::Event<'_, ()>) -> Event {
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
                        let key = Self::get_input_key(key_code);
                        match state {
                            ElementState::Pressed => status = Event::KeyPressed(key),
                            ElementState::Released => status = Event::KeyReleased(key),
                        }
                    }
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                            scroll as f32
                        }
                    };
                    status = Event::MouseWheel(delta);
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => {
                    status = match state {
                        ElementState::Pressed => Event::MousePressed,
                        ElementState::Released => Event::MouseReleased,
                    }
                }
                _ => (),
            },
            winit::event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => status = Event::MouseMotion(delta.0, delta.1),
            winit::event::Event::RedrawRequested(_) => status = Event::Redraw,
            winit::event::Event::MainEventsCleared {} => status = Event::Cleared,
            _ => (),
        }

        debug!("Status={:?}", status);
        status
    }

    fn get_input_key(key_code: VirtualKeyCode) -> Key {
        match key_code {
            VirtualKeyCode::Left => Key::Left,
            VirtualKeyCode::Right => Key::Right,
            VirtualKeyCode::Up => Key::Up,
            VirtualKeyCode::Down => Key::Down,
            VirtualKeyCode::PageUp => Key::PageUp,
            VirtualKeyCode::PageDown => Key::PageDown,
            VirtualKeyCode::Return => Key::Return,
            VirtualKeyCode::LShift => Key::LShift,
            VirtualKeyCode::Space => Key::Space,
            VirtualKeyCode::Tab => Key::Tab,
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            _ => Key::Unknown,
        }
    }
}
