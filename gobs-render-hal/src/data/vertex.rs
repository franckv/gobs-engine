use bitflags::bitflags;
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use gobs_core::{Color, Transform};
use gobs_vulkan::pipelines::VertexAttributeFormat;

use crate::{
    AttributeData,
    data::{AlignMode, Attribute},
};

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
    fn idx(self) -> usize {
        self.bits().trailing_zeros() as usize
    }

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

    pub fn offset_of(self, attr: VertexAttribute, mode: AlignMode) -> Option<usize> {
        let offsets = Attribute::offsets(&self.attributes(), mode);

        self.iter()
            .zip(offsets)
            .find(|(a, _)| *a == attr)
            .map(|(_, offset)| offset)
    }

    pub fn size(&self, mode: AlignMode) -> usize {
        Attribute::stride(&self.attributes(), mode)
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

#[derive(Clone, Debug, Serialize)]
pub struct VertexData {
    data: Vec<AttributeData>,
}

impl VertexData {
    pub fn builder() -> VertexDataBuilder {
        VertexDataBuilder::new()
    }

    pub fn position(&self) -> Vec3 {
        match self.data[VertexAttribute::POSITION.idx()] {
            AttributeData::Vec3F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.data[VertexAttribute::POSITION.idx()] = AttributeData::Vec3F(position.into())
    }

    pub fn color(&self) -> Color {
        match self.data[VertexAttribute::COLOR.idx()] {
            AttributeData::Vec4F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.data[VertexAttribute::COLOR.idx()] = AttributeData::Vec4F(color.into())
    }

    pub fn texture(&self) -> Vec2 {
        match self.data[VertexAttribute::TEXTURE.idx()] {
            AttributeData::Vec2F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn set_texture(&mut self, texture: Vec2) {
        self.data[VertexAttribute::TEXTURE.idx()] = AttributeData::Vec2F(texture.into())
    }

    pub fn normal(&self) -> Vec3 {
        match self.data[VertexAttribute::NORMAL.idx()] {
            AttributeData::Vec3F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn set_normal(&mut self, normal: Vec3) {
        self.data[VertexAttribute::NORMAL.idx()] = AttributeData::Vec3F(normal.into())
    }

    pub fn normal_texture(&self) -> Vec2 {
        match self.data[VertexAttribute::NORMAL_TEXTURE.idx()] {
            AttributeData::Vec2F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn tangent(&self) -> Vec3 {
        match self.data[VertexAttribute::TANGENT.idx()] {
            AttributeData::Vec3F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn set_tangent(&mut self, tangent: Vec3) {
        self.data[VertexAttribute::TANGENT.idx()] = AttributeData::Vec3F(tangent.into())
    }

    pub fn bitangent(&self) -> Vec3 {
        match self.data[VertexAttribute::BITANGENT.idx()] {
            AttributeData::Vec3F(v) => v.into(),
            _ => unreachable!(),
        }
    }

    pub fn set_bitangent(&mut self, bitangent: Vec3) {
        self.data[VertexAttribute::BITANGENT.idx()] = AttributeData::Vec3F(bitangent.into())
    }

    pub fn transform(&self, transform: Transform) -> VertexData {
        let mut vertex = self.clone();

        vertex.set_position(transform * vertex.position());

        vertex
    }

    fn get_bytes(&self, flag: VertexAttribute, data: &mut Vec<u8>) {
        self.data[flag.idx()].copy(data);
    }

    pub fn copy_data(
        vertices: &[VertexData],
        flags: VertexAttribute,
        data: &mut Vec<u8>,
        mode: AlignMode,
    ) {
        let attributes = flags.attributes();
        let offsets = Attribute::offsets(&attributes, mode);
        let size = Attribute::stride(&attributes, mode);

        for vertex in vertices {
            let data_start = data.len();
            for (flag, offset) in flags.iter().zip(&offsets) {
                let delta = data.len() - data_start;
                AttributeData::pad(data, offset - delta);

                vertex.get_bytes(flag, data);
            }
            let delta = data.len() - data_start;
            AttributeData::pad(data, size - delta);
        }
    }
}

/*
impl Add<VertexData> for VertexData {
    type Output = Self;

    fn add(self, rhs: VertexData) -> Self::Output {
        VertexData {
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
*/

pub struct VertexDataBuilder {
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
            position: None,
            color: None,
            texture: None,
            normal: None,
            normal_texture: None,
            tangent: None,
            bitangent: None,
        }
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
        let data = vec![
            AttributeData::Vec3F(self.position.unwrap_or(Vec3::splat(0.)).into()),
            AttributeData::Vec4F(self.color.unwrap_or(Color::WHITE).into()),
            AttributeData::Vec2F(self.texture.unwrap_or(Vec2::splat(0.)).into()),
            AttributeData::Vec3F(self.normal.unwrap_or(Vec3::splat(0.)).into()),
            AttributeData::Vec2F(self.normal_texture.unwrap_or(Vec2::splat(0.)).into()),
            AttributeData::Vec3F(self.tangent.unwrap_or(Vec3::splat(0.)).into()),
            AttributeData::Vec3F(self.bitangent.unwrap_or(Vec3::splat(0.)).into()),
        ];

        debug_assert_eq!(data.len(), VertexAttribute::all().iter().count());

        VertexData { data }
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

        let mode = AlignMode::Scalar;

        let vertex_attributes = VertexAttribute::POSITION;
        let offsets = Attribute::offsets(&vertex_attributes.attributes(), mode);
        assert_eq!(vertex_attributes.size(mode), 12);
        assert_eq!(offsets[0], 0);

        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;
        let offsets = Attribute::offsets(&vertex_attributes.attributes(), mode);
        assert_eq!(vertex_attributes.size(mode), 28);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 12);

        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::COLOR
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;
        let _offsets = Attribute::offsets(&vertex_attributes.attributes(), mode);
        assert_eq!(vertex_attributes.size(mode), 72);

        let vertex_attributes = VertexAttribute::TEXTURE | VertexAttribute::NORMAL_TEXTURE;
        let _offsets = Attribute::offsets(&vertex_attributes.attributes(), mode);
        assert_eq!(vertex_attributes.size(mode), 16);

        let vertex_attributes =
            VertexAttribute::NORMAL | VertexAttribute::TANGENT | VertexAttribute::BITANGENT;
        let _offsets = Attribute::offsets(&vertex_attributes.attributes(), mode);
        assert_eq!(vertex_attributes.size(mode), 36);
    }
}
