use winit;
use winit::EventsLoop;
use winit::Event::WindowEvent;
use winit::{ElementState, KeyboardInput, VirtualKeyCode};

use input::{Key,InputMap};

pub enum Event {
    Resize,
    Close,
    Continue
}

pub struct InputHandler {
    events_loop: EventsLoop,
    input_map: InputMap
}

impl InputHandler {
    pub fn new(events_loop: EventsLoop) -> Self {
        InputHandler {
            events_loop: events_loop,
            input_map: InputMap::new()
        }
    }

    pub fn get_input_map(&self) -> &InputMap {
        &self.input_map
    }

    pub fn read_inputs(&mut self) -> Event {
        let mut status = Event::Continue;

        let input_map = &mut self.input_map;

        input_map.reset();

        self.events_loop.poll_events(|event| {
            match event {
                WindowEvent { event, .. } => match event {
                    winit::WindowEvent::Closed => status = Event::Close,
                    winit::WindowEvent::Resized(_, _) => status = Event::Resize,
                    winit::WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(key_code),
                            state, ..
                        }, ..
                    } => match key_code {
                        VirtualKeyCode::Escape => status = Event::Close,
                        _ => {
                            let key = Self::get_input_key(key_code);
                            match state {
                                ElementState::Pressed => input_map.key_press(key),
                                ElementState::Released => input_map.key_release(key)
                            }
                        },
                    },
                    _ => ()
                },
                _ => ()
            }
        });

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
            _ => Key::Unknown
        }
    }
}
