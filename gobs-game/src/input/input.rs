#[derive(Copy, Clone)]
pub enum Key {
    Left = 0,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Return,
    Space,
    Tab,
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Unknown
}

pub struct InputMap {
    state: [bool; 1 + Key::Unknown as usize],
    previous: [bool; 1 + Key::Unknown as usize]
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            state: [false; 1 + Key::Unknown as usize],
            previous: [false; 1 + Key::Unknown as usize]
        }
    }

    pub fn key_press(&mut self, key: Key) {
        self.state[key as usize] = true;
    }

    pub fn key_release(&mut self, key: Key) {
        self.state[key as usize] = false;
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.state[key as usize] && !self.previous[key as usize]
    }

    pub fn reset(&mut self) {
        self.previous = self.state;
        self.state = [false; 1 + Key::Unknown as usize];
    }
}
