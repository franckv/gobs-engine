use std::ops::{Add, Mul};

use bitflags::bitflags;
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use gobs_core::{Color, Transform};
use gobs_vulkan::pipelines::VertexAttributeFormat;

use crate::data::{AlignMode, Attribute};

bitflags! {
    #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    fn attribute_type(self) -> Attribute {
        match self {
            VertexAttribute::POSITION => Attribute::Vec3F,
            VertexAttribute::COLOR => Attribute::Vec4F,
            VertexAttribute::TEXTURE => Attribute::Vec2F,
            VertexAttribute::NORMAL => Attribute::Vec3F,
            VertexAttribute::NORMAL_TEXTURE => Attribute::Vec2F,
            VertexAttribute::TANGENT => Attribute::Vec3F,
            VertexAttribute::BITANGENT => Attribute::Vec3F,
            _ => unimplemented!(),
        }
    }

    fn attributes(self) -> Vec<Attribute> {
        self.iter().map(|v| v.attribute_type()).collect()
    }

    fn attribute_offsets(self, mode: AlignMode) -> Vec<(VertexAttribute, usize)> {
        let offsets = Attribute::offsets(&self.attributes(), mode);

        self.iter().zip(offsets).collect()
    }

    pub fn offset_of(self, attr: VertexAttribute, mode: AlignMode) -> usize {
        let offsets = VertexAttribute::attribute_offsets(self, mode);

        offsets.iter().find(|(a, _)| *a == attr).unwrap().1
    }

    pub fn size(&self) -> usize {
        Attribute::stride(&self.attributes(), AlignMode::Compact)
    }
}

impl From<VertexAttribute> for VertexAttributeFormat {
    fn from(value: VertexAttribute) -> Self {
        match value {
            VertexAttribute::POSITION => VertexAttributeFormat::Vec3,
            VertexAttribute::COLOR => VertexAttributeFormat::Vec4,
            VertexAttribute::TEXTURE => VertexAttributeFormat::Vec2,
            VertexAttribute::NORMAL => VertexAttributeFormat::Vec3,
            VertexAttribute::NORMAL_TEXTURE => VertexAttributeFormat::Vec2,
            VertexAttribute::TANGENT => VertexAttributeFormat::Vec3,
            VertexAttribute::BITANGENT => VertexAttributeFormat::Vec3,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct VertexData {
    pub layout: AlignMode,
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

    pub fn transform(&self, transform: Transform) -> VertexData {
        let mut vertex = *self;

        vertex.position = transform * vertex.position;

        vertex
    }

    fn get_bytes(&self, flag: VertexAttribute, data: &mut Vec<u8>) {
        match flag {
            VertexAttribute::POSITION => {
                data.extend_from_slice(bytemuck::cast_slice(&self.position.to_array()))
            }
            VertexAttribute::COLOR => {
                data.extend_from_slice(bytemuck::cast_slice(&Into::<[f32; 4]>::into(self.color)))
            }
            VertexAttribute::TEXTURE => {
                data.extend_from_slice(bytemuck::cast_slice(&self.texture.to_array()))
            }
            VertexAttribute::NORMAL => {
                data.extend_from_slice(bytemuck::cast_slice(&self.normal.to_array()))
            }
            VertexAttribute::NORMAL_TEXTURE => {
                data.extend_from_slice(bytemuck::cast_slice(&self.normal_texture.to_array()))
            }
            VertexAttribute::TANGENT => {
                data.extend_from_slice(bytemuck::cast_slice(&self.tangent.to_array()))
            }
            VertexAttribute::BITANGENT => {
                data.extend_from_slice(bytemuck::cast_slice(&self.bitangent.to_array()))
            }
            _ => unimplemented!(),
        }
    }

    pub fn copy_data(vertices: &[VertexData], flags: VertexAttribute, data: &mut Vec<u8>) {
        let offsets = flags.attribute_offsets(AlignMode::Compact);

        for vertex in vertices {
            let data_start = data.len();
            for (flag, offset) in &offsets {
                let delta = data.len() - data_start;
                Self::pad(data, offset - delta);

                vertex.get_bytes(*flag, data);
            }
        }
    }

    fn pad(data: &mut Vec<u8>, len: usize) {
        for _ in 0..len {
            data.push(0_u8);
        }
    }
}

impl Add<VertexData> for VertexData {
    type Output = Self;

    fn add(self, rhs: VertexData) -> Self::Output {
        VertexData {
            layout: self.layout,
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
            layout: self.layout,
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
    pub layout: AlignMode,
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
            layout: AlignMode::Compact,
            position: None,
            color: None,
            texture: None,
            normal: None,
            normal_texture: None,
            tangent: None,
            bitangent: None,
        }
    }

    pub fn layout(mut self, layout: AlignMode) -> Self {
        self.layout = layout;

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
            layout: self.layout,
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

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::{
        VertexAttribute,
        data::{AlignMode, Attribute},
    };

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_vertex_align() {
        setup();

        let vertex_attributes = VertexAttribute::POSITION;
        let offsets = Attribute::offsets(&vertex_attributes.attributes(), AlignMode::Compact);
        assert_eq!(vertex_attributes.size(), 12);
        assert_eq!(offsets[0], 0);

        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;
        let offsets = Attribute::offsets(&vertex_attributes.attributes(), AlignMode::Compact);
        assert_eq!(vertex_attributes.size(), 28);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 12);

        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::COLOR
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;
        let _offsets = Attribute::offsets(&vertex_attributes.attributes(), AlignMode::Compact);
        assert_eq!(vertex_attributes.size(), 72);

        let vertex_attributes = VertexAttribute::TEXTURE | VertexAttribute::NORMAL_TEXTURE;
        let _offsets = Attribute::offsets(&vertex_attributes.attributes(), AlignMode::Compact);
        assert_eq!(vertex_attributes.size(), 16);

        let vertex_attributes =
            VertexAttribute::NORMAL | VertexAttribute::TANGENT | VertexAttribute::BITANGENT;
        let _offsets = Attribute::offsets(&vertex_attributes.attributes(), AlignMode::Compact);
        assert_eq!(vertex_attributes.size(), 36);
    }
}
