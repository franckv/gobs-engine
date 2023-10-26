use bitflags::bitflags;
use glam::{Vec2, Vec3};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VertexFlag: u32 {
        const POSITION = 1;
        const TEXTURE = 1 << 1;
        const NORMAL = 1 << 2;
        const INDEX = 1 << 3;
    }
}

pub struct VertexDataBuilder {
    flags: VertexFlag,
    pub position: Option<Vec3>,
    pub texture: Option<Vec2>,
    pub normal: Option<Vec3>,
    pub tangent: Option<Vec3>,
    pub bitangent: Option<Vec3>,
    pub index: Option<f32>,
}

impl VertexDataBuilder {
    pub fn position(mut self, position: Vec3) -> Self {
        self.position = Some(position);

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

    pub fn tangent(mut self, tangent: Vec3) -> Self {
        self.tangent = Some(tangent);

        self
    }

    pub fn bitangent(mut self, bitangent: Vec3) -> Self {
        self.bitangent = Some(bitangent);

        self
    }

    pub fn index(mut self, index: f32) -> Self {
        self.index = Some(index);

        self
    }

    pub fn build(self) -> VertexData {
        VertexData {
            flags: self.flags,
            position: self.position.unwrap_or(Vec3::splat(0.)),
            texture: self.texture.unwrap_or(Vec2::splat(0.)),
            normal: self.normal.unwrap_or(Vec3::splat(0.)),
            tangent: self.tangent.unwrap_or(Vec3::splat(0.)),
            bitangent: self.bitangent.unwrap_or(Vec3::splat(0.)),
            index: self.index.unwrap_or(1.),
        }
    }
}

#[derive(Clone)]
pub struct VertexData {
    flags: VertexFlag,
    pub position: Vec3,
    pub texture: Vec2,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub index: f32,
}

impl VertexData {
    pub fn new(flags: VertexFlag) -> VertexDataBuilder {
        VertexDataBuilder {
            flags,
            position: None,
            texture: None,
            normal: None,
            tangent: None,
            bitangent: None,
            index: None,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn texture(&self) -> Vec2 {
        self.texture
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    pub fn tangent(&self) -> Vec3 {
        self.tangent
    }

    pub fn bitangent(&self) -> Vec3 {
        self.bitangent
    }

    pub fn index(&self) -> f32 {
        self.index
    }

    pub fn set_tangent(&mut self, tangent: Vec3) {
        self.tangent = tangent
    }

    pub fn set_bitangent(&mut self, bitangent: Vec3) {
        self.bitangent = bitangent
    }

    pub fn raw(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        if self.flags.contains(VertexFlag::POSITION) {
            data.extend_from_slice(bytemuck::cast_slice(&self.position.to_array()));
        };

        if self.flags.contains(VertexFlag::TEXTURE) {
            data.extend_from_slice(bytemuck::cast_slice(&self.texture.to_array()));
        };

        if self.flags.contains(VertexFlag::NORMAL) {
            data.extend_from_slice(bytemuck::cast_slice(&self.normal.to_array()));
            data.extend_from_slice(bytemuck::cast_slice(&self.tangent.to_array()));
            data.extend_from_slice(bytemuck::cast_slice(&self.bitangent.to_array()));
        };

        if self.flags.contains(VertexFlag::INDEX) {
            data.extend_from_slice(bytemuck::cast_slice(&[self.index]));
        };

        data
    }

    pub fn size(flags: VertexFlag) -> usize {
        flags
            .iter()
            .map(|bit| match bit {
                VertexFlag::POSITION => std::mem::size_of::<Vec3>(),
                VertexFlag::TEXTURE => std::mem::size_of::<Vec2>(),
                VertexFlag::NORMAL => 3 * std::mem::size_of::<Vec3>(),
                VertexFlag::INDEX => std::mem::size_of::<f32>(),
                _ => unimplemented!(),
            })
            .sum()
    }
}
