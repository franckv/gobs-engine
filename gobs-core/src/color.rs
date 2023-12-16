use std::ops::{Add, Div, Mul};

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use image::Rgba;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub const RED: Color = Color {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const GREEN: Color = Color {
        r: 0.,
        g: 1.,
        b: 0.,
        a: 1.,
    };
    pub const BLUE: Color = Color {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const YELLOW: Color = Color {
        r: 1.,
        g: 1.,
        b: 0.,
        a: 1.,
    };
    pub const CYAN: Color = Color {
        r: 0.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const MAGENTA: Color = Color {
        r: 1.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const WHITE: Color = Color {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const BLACK: Color = Color {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const GREY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        )
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a,
        }
    }
}

impl Div<f32> for Color {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
            a: self.a,
        }
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        [
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        ]
    }
}

impl Into<Rgba<f32>> for Color {
    fn into(self) -> Rgba<f32> {
        image::Rgba([self.r, self.g, self.b, self.a])
    }
}

impl Into<Rgba<u8>> for Color {
    fn into(self) -> Rgba<u8> {
        image::Rgba([
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        ])
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Color {
            r: value.x,
            g: value.y,
            b: value.z,
            a: 1.,
        }
    }
}
