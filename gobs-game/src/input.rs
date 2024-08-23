use winit::event::{self};
use winit::keyboard::{self};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    Left = 0,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Return,
    Space,
    LShift,
    Tab,
    Plus,
    Minus,
    Backspace,
    Equals,
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Unknown,
}

impl From<keyboard::Key> for Key {
    fn from(key_code: keyboard::Key) -> Key {
        match key_code {
            keyboard::Key::Named(named_key) => match named_key {
                keyboard::NamedKey::ArrowLeft => Key::Left,
                keyboard::NamedKey::ArrowRight => Key::Right,
                keyboard::NamedKey::ArrowUp => Key::Up,
                keyboard::NamedKey::ArrowDown => Key::Down,
                keyboard::NamedKey::PageUp => Key::PageUp,
                keyboard::NamedKey::PageDown => Key::PageDown,
                keyboard::NamedKey::Enter => Key::Return,
                keyboard::NamedKey::Shift => Key::LShift,
                keyboard::NamedKey::Space => Key::Space,
                keyboard::NamedKey::Tab => Key::Tab,
                keyboard::NamedKey::Backspace => Key::Backspace,
                _ => Key::Unknown,
            },
            keyboard::Key::Character(c) => match c.to_uppercase().as_str() {
                "0" => Key::N0,
                "1" => Key::N1,
                "2" => Key::N2,
                "3" => Key::N3,
                "4" => Key::N4,
                "5" => Key::N5,
                "6" => Key::N6,
                "7" => Key::N7,
                "8" => Key::N8,
                "9" => Key::N9,
                "A" => Key::A,
                "B" => Key::B,
                "C" => Key::C,
                "D" => Key::D,
                "E" => Key::E,
                "F" => Key::F,
                "G" => Key::G,
                "H" => Key::H,
                "I" => Key::I,
                "J" => Key::J,
                "K" => Key::K,
                "L" => Key::L,
                "M" => Key::M,
                "N" => Key::N,
                "O" => Key::O,
                "P" => Key::P,
                "Q" => Key::Q,
                "R" => Key::R,
                "S" => Key::S,
                "T" => Key::T,
                "U" => Key::U,
                "V" => Key::V,
                "W" => Key::W,
                "X" => Key::X,
                "Y" => Key::Y,
                "Z" => Key::Z,
                "+" => Key::Plus,
                "-" => Key::Minus,
                "=" => Key::Equals,
                _ => Key::Unknown,
            },
            _ => Key::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Unknown,
}

impl From<event::MouseButton> for MouseButton {
    fn from(value: event::MouseButton) -> Self {
        match value {
            event::MouseButton::Left => MouseButton::Left,
            event::MouseButton::Right => MouseButton::Right,
            event::MouseButton::Middle => MouseButton::Middle,
            event::MouseButton::Back => MouseButton::Unknown,
            event::MouseButton::Forward => MouseButton::Unknown,
            event::MouseButton::Other(_) => MouseButton::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Input {
    CursorMoved(f64, f64),
    KeyPressed(Key),
    KeyReleased(Key),
    MouseWheel(f32),
    MouseMotion(f64, f64),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
}
