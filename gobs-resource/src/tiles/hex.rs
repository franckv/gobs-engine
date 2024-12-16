use std::{
    collections::HashMap,
    ops::{Add, Mul},
};

pub enum HexDirection {
    N,
    NE,
    SE,
    S,
    SW,
    NW,
}

#[derive(Hash, PartialEq, Eq)]
pub struct HexPos {
    q: i32,
    r: i32,
}

impl HexPos {
    pub const N: HexPos = HexPos { q: 0, r: -1 };
    pub const NE: HexPos = HexPos { q: 1, r: -1 };
    pub const SE: HexPos = HexPos { q: 1, r: 0 };
    pub const S: HexPos = HexPos { q: 0, r: 1 };
    pub const SW: HexPos = HexPos { q: -1, r: 1 };
    pub const NW: HexPos = HexPos { q: -1, r: 0 };

    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
}

impl Add for HexPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        HexPos {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl Mul<i32> for HexPos {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        HexPos {
            q: rhs * self.q,
            r: rhs * self.r,
        }
    }
}

impl Mul<HexPos> for i32 {
    type Output = HexPos;

    fn mul(self, rhs: HexPos) -> Self::Output {
        HexPos {
            q: self * rhs.q,
            r: self * rhs.r,
        }
    }
}

pub struct HexMap<T> {
    tiles: HashMap<HexPos, T>,
}

impl<T> HexMap<T> {
    pub fn get(&self, pos: HexPos) -> Option<&T> {
        self.tiles.get(&pos)
    }

    pub fn insert(&mut self, pos: HexPos, tile: T) {
        self.tiles.insert(pos, tile);
    }
}
