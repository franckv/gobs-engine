use bitflags::bitflags;
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use gobs_core::{Color, Transform, data::data_buffer::DataBuffer};
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
    const SIZE: usize = Self::all().bits().count_ones() as usize;

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
    data: [AttributeData; VertexAttribute::SIZE],
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

    fn get_bytes<B>(&self, flag: VertexAttribute, data: &mut B)
    where
        B: DataBuffer,
    {
        self.data[flag.idx()].copy(data);
    }

    pub fn copy_data<B>(
        vertices: &[VertexData],
        flags: VertexAttribute,
        data: &mut B,
        mode: AlignMode,
    ) where
        B: DataBuffer,
    {
        let attributes = flags.attributes();
        let offsets = Attribute::offsets(&attributes, mode);
        let size = Attribute::stride(&attributes, mode);

        for vertex in vertices {
            let data_start = data.len();
            for (flag, offset) in flags.iter().zip(&offsets) {
                let delta = data.len() - data_start;
                data.pad(offset - delta);

                vertex.get_bytes(flag, data);
            }
            let delta = data.len() - data_start;
            data.pad(size - delta);
        }
    }
}

pub struct VertexDataBuilder {
    pub position: Vec3,
    pub color: Color,
    pub texture: Vec2,
    pub normal: Vec3,
    pub normal_texture: Vec2,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl VertexDataBuilder {
    fn new() -> Self {
        VertexDataBuilder {
            position: Vec3::splat(0.),
            color: Color::WHITE,
            texture: Vec2::splat(0.),
            normal: Vec3::splat(0.),
            normal_texture: Vec2::splat(0.),
            tangent: Vec3::splat(0.),
            bitangent: Vec3::splat(0.),
        }
    }

    pub fn position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;

        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;

        self
    }

    pub fn texture(&mut self, texture: Vec2) -> &mut Self {
        self.texture = texture;

        self
    }

    pub fn normal(&mut self, normal: Vec3) -> &mut Self {
        self.normal = normal;

        self
    }

    pub fn normal_texture(&mut self, normal_texture: Vec2) -> &mut Self {
        self.normal_texture = normal_texture;

        self
    }

    pub fn tangent(&mut self, tangent: Vec3) -> &mut Self {
        self.tangent = tangent;

        self
    }

    pub fn bitangent(&mut self, bitangent: Vec3) -> &mut Self {
        self.bitangent = bitangent;

        self
    }

    pub fn build(&mut self) -> VertexData {
        let data = [
            AttributeData::Vec3F(self.position.into()),
            AttributeData::Vec4F(self.color.into()),
            AttributeData::Vec2F(self.texture.into()),
            AttributeData::Vec3F(self.normal.into()),
            AttributeData::Vec2F(self.normal_texture.into()),
            AttributeData::Vec3F(self.tangent.into()),
            AttributeData::Vec3F(self.bitangent.into()),
        ];

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
