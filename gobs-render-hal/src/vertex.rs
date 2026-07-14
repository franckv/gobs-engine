use std::ops::{Add, Mul};

use bitflags::bitflags;
use glam::{Vec2, Vec3};

use gobs_core::{Color, Transform};
use serde::{Deserialize, Serialize};

const POS_SIZE: usize = 12;
const COLOR_SIZE: usize = 16;
const TEX_SIZE: usize = 8;
const NORMAL_SIZE: usize = 12;
const NORMAL_TEX_SIZE: usize = 8;
const TANGENT_SIZE: usize = 12;
const BITANGENT_SIZE: usize = 12;

const POS_ALIGN: usize = 16;
const COLOR_ALIGN: usize = 16;
const TEX_ALIGN: usize = 8;
const NORMAL_ALIGN: usize = 16;
const NORMAL_TEX_ALIGN: usize = 8;
const TANGENT_ALIGN: usize = 16;
const BITANGENT_ALIGN: usize = 16;

bitflags! {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[serde(transparent)]
    pub struct VertexAttribute: u32 {
        const POSITION = 1;
        const COLOR = 1 << 1;
        const TEXTURE = 1 << 2;
        const NORMAL = 1 << 3;
        const NORMAL_TEXTURE = 1 << 4;
        const TANGENT = 1 << 5;
        const BITANGENT = 1 << 6;
    }
}

impl VertexAttribute {
    pub fn alignment(&self) -> usize {
        let mut align = 0;
        for bit in self.iter() {
            let bit_align = match bit {
                VertexAttribute::POSITION => POS_ALIGN,
                VertexAttribute::COLOR => COLOR_ALIGN,
                VertexAttribute::TEXTURE => TEX_ALIGN,
                VertexAttribute::NORMAL => NORMAL_ALIGN,
                VertexAttribute::NORMAL_TEXTURE => NORMAL_TEX_ALIGN,
                VertexAttribute::TANGENT => TANGENT_ALIGN,
                VertexAttribute::BITANGENT => BITANGENT_ALIGN,
                _ => 0,
            };

            if bit_align > align {
                align = bit_align;
            }
        }

        align
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        for bit in self.iter() {
            let bit_align = match bit {
                VertexAttribute::POSITION => POS_SIZE,
                VertexAttribute::COLOR => COLOR_SIZE,
                VertexAttribute::TEXTURE => TEX_SIZE,
                VertexAttribute::NORMAL => NORMAL_SIZE,
                VertexAttribute::NORMAL_TEXTURE => NORMAL_TEX_SIZE,
                VertexAttribute::TANGENT => TANGENT_SIZE,
                VertexAttribute::BITANGENT => BITANGENT_SIZE,
                _ => 0,
            };

            size += bit_align;
        }

        size
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct VertexData {
    pub padding: bool, // false for vertex buffer, true for storage buffer
    pub position: Vec3,
    pub color: Color,
    pub texture: Vec2,
    pub normal: Vec3,
    pub normal_texture: Vec2,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl VertexData {
    pub fn builder() -> VertexDataBuilder {
        VertexDataBuilder::new()
    }

    pub fn raw(&self, flags: &VertexAttribute, alignment: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        self.copy_data(flags, alignment, &mut data);

        data
    }

    pub fn transform(&self, transform: Transform) -> VertexData {
        let mut vertex = *self;

        vertex.position = transform * vertex.position;

        vertex
    }

    pub fn copy_data(&self, flags: &VertexAttribute, alignment: usize, data: &mut Vec<u8>) {
        if flags.contains(VertexAttribute::POSITION) {
            data.extend_from_slice(bytemuck::cast_slice(&self.position.to_array()));
            Self::pad(data, self.padding, alignment - POS_SIZE);
        };

        if flags.contains(VertexAttribute::COLOR) {
            data.extend_from_slice(bytemuck::cast_slice(&Into::<[f32; 4]>::into(self.color)));
            Self::pad(data, self.padding, alignment - COLOR_SIZE);
        };

        if flags.contains(VertexAttribute::TEXTURE) {
            data.extend_from_slice(bytemuck::cast_slice(&self.texture.to_array()));
            Self::pad(data, self.padding, alignment - TEX_SIZE);
        };

        if flags.contains(VertexAttribute::NORMAL) {
            data.extend_from_slice(bytemuck::cast_slice(&self.normal.to_array()));
            Self::pad(data, self.padding, alignment - NORMAL_SIZE);
        };

        if flags.contains(VertexAttribute::NORMAL_TEXTURE) {
            data.extend_from_slice(bytemuck::cast_slice(&self.normal_texture.to_array()));
            Self::pad(data, self.padding, alignment - NORMAL_TEX_SIZE);
        };

        if flags.contains(VertexAttribute::TANGENT) {
            data.extend_from_slice(bytemuck::cast_slice(&self.tangent.to_array()));
            Self::pad(data, self.padding, alignment - TANGENT_SIZE);
        };

        if flags.contains(VertexAttribute::BITANGENT) {
            data.extend_from_slice(bytemuck::cast_slice(&self.bitangent.to_array()));
            Self::pad(data, self.padding, alignment - BITANGENT_SIZE);
        };
    }

    fn pad(data: &mut Vec<u8>, padding: bool, len: usize) {
        if padding {
            for _ in 0..len {
                data.push(0_u8);
            }
        }
    }

    pub fn size(flags: VertexAttribute, padding: bool) -> usize {
        flags
            .iter()
            .map(|bit| match bit {
                VertexAttribute::POSITION
                | VertexAttribute::COLOR
                | VertexAttribute::TEXTURE
                | VertexAttribute::NORMAL
                | VertexAttribute::NORMAL_TEXTURE
                | VertexAttribute::TANGENT
                | VertexAttribute::BITANGENT => {
                    if padding {
                        flags.alignment()
                    } else {
                        bit.size()
                    }
                }
                _ => unimplemented!(),
            })
            .sum()
    }
}

impl Add<VertexData> for VertexData {
    type Output = Self;

    fn add(self, rhs: VertexData) -> Self::Output {
        VertexData {
            padding: self.padding,
            position: self.position + rhs.position,
            color: self.color + rhs.color,
            texture: self.texture + rhs.texture,
            normal: self.normal + rhs.normal,
            normal_texture: self.normal_texture + rhs.normal_texture,
            tangent: self.tangent + rhs.tangent,
            bitangent: self.bitangent + rhs.bitangent,
        }
    }
}

impl Mul<f32> for VertexData {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        VertexData {
            padding: self.padding,
            position: self.position * rhs,
            color: self.color * rhs,
            texture: self.texture * rhs,
            normal: self.normal * rhs,
            normal_texture: self.normal_texture * rhs,
            tangent: self.tangent * rhs,
            bitangent: self.bitangent * rhs,
        }
    }
}

pub struct VertexDataBuilder {
    pub padding: Option<bool>,
    pub position: Option<Vec3>,
    pub color: Option<Color>,
    pub texture: Option<Vec2>,
    pub normal: Option<Vec3>,
    pub normal_texture: Option<Vec2>,
    pub tangent: Option<Vec3>,
    pub bitangent: Option<Vec3>,
}

impl VertexDataBuilder {
    fn new() -> Self {
        VertexDataBuilder {
            padding: None,
            position: None,
            color: None,
            texture: None,
            normal: None,
            normal_texture: None,
            tangent: None,
            bitangent: None,
        }
    }

    pub fn padding(mut self, padding: bool) -> Self {
        self.padding = Some(padding);

        self
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = Some(position);

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);

        self
    }

    pub fn texture(mut self, texture: Vec2) -> Self {
        self.texture = Some(texture);

        self
    }

    pub fn normal(mut self, normal: Vec3) -> Self {
        self.normal = Some(normal);

        self
    }

    pub fn normal_texture(mut self, normal_texture: Vec2) -> Self {
        self.normal_texture = Some(normal_texture);

        self
    }

    pub fn tangent(mut self, tangent: Vec3) -> Self {
        self.tangent = Some(tangent);

        self
    }

    pub fn bitangent(mut self, bitangent: Vec3) -> Self {
        self.bitangent = Some(bitangent);

        self
    }

    pub fn build(self) -> VertexData {
        VertexData {
            padding: self.padding.expect("Missing padding"),
            position: self.position.unwrap_or(Vec3::splat(0.)),
            color: self.color.unwrap_or(Color::WHITE),
            texture: self.texture.unwrap_or(Vec2::splat(0.)),
            normal: self.normal.unwrap_or(Vec3::splat(0.)),
            normal_texture: self.normal_texture.unwrap_or(Vec2::splat(0.)),
            tangent: self.tangent.unwrap_or(Vec3::splat(0.)),
            bitangent: self.bitangent.unwrap_or(Vec3::splat(0.)),
        }
    }
}

impl Default for VertexDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
