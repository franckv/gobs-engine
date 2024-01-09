use bitflags::bitflags;
use glam::{Vec2, Vec3, Vec4};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VertexFlag: u32 {
        const POSITION = 1;
        const COLOR = 1 << 1;
        const TEXTURE = 1 << 2;
        const NORMAL = 1 << 3;
        const NORMAL_TEXTURE = 1 << 4;
        const TANGENT = 1 << 5;
        const BITANGENT = 1 << 6;
    }
}

impl VertexFlag {
    pub fn alignment(&self) -> usize {
        let mut align = 0;
        for bit in self.iter() {
            let bit_align = match bit {
                VertexFlag::POSITION => 16,
                VertexFlag::COLOR => 16,
                VertexFlag::TEXTURE => 8,
                VertexFlag::NORMAL => 16,
                VertexFlag::NORMAL_TEXTURE => 8,
                VertexFlag::TANGENT => 16,
                VertexFlag::BITANGENT => 16,
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
                VertexFlag::POSITION => 12,
                VertexFlag::COLOR => 16,
                VertexFlag::TEXTURE => 8,
                VertexFlag::NORMAL => 12,
                VertexFlag::NORMAL_TEXTURE => 8,
                VertexFlag::TANGENT => 12,
                VertexFlag::BITANGENT => 12,
                _ => 0,
            };

            size += bit_align;
        }

        size
    }
}

#[derive(Clone, Copy)]
pub struct VertexData {
    pub position: Vec3,
    pub color: Vec4,
    pub texture: Vec2,
    pub normal: Vec3,
    pub normal_texture: Vec2,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub index: f32,
}

impl VertexData {
    pub fn builder() -> VertexDataBuilder {
        VertexDataBuilder::new()
    }

    pub fn raw(&self, flags: VertexFlag) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        if flags.contains(VertexFlag::POSITION) {
            data.extend_from_slice(bytemuck::cast_slice(&self.position.to_array()));

            let align = flags.alignment() - VertexFlag::POSITION.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        if flags.contains(VertexFlag::COLOR) {
            data.extend_from_slice(bytemuck::cast_slice(&self.color.to_array()));

            let align = flags.alignment() - VertexFlag::COLOR.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        if flags.contains(VertexFlag::TEXTURE) {
            data.extend_from_slice(bytemuck::cast_slice(&self.texture.to_array()));

            let align = flags.alignment() - VertexFlag::TEXTURE.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        if flags.contains(VertexFlag::NORMAL) {
            data.extend_from_slice(bytemuck::cast_slice(&self.normal.to_array()));

            let align = flags.alignment() - VertexFlag::NORMAL.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        if flags.contains(VertexFlag::NORMAL_TEXTURE) {
            data.extend_from_slice(bytemuck::cast_slice(&self.normal_texture.to_array()));

            let align = flags.alignment() - VertexFlag::NORMAL_TEXTURE.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        if flags.contains(VertexFlag::TANGENT) {
            data.extend_from_slice(bytemuck::cast_slice(&self.tangent.to_array()));

            let align = flags.alignment() - VertexFlag::TANGENT.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        if flags.contains(VertexFlag::BITANGENT) {
            data.extend_from_slice(bytemuck::cast_slice(&self.bitangent.to_array()));

            let align = flags.alignment() - VertexFlag::BITANGENT.size();
            for _ in 0..align {
                data.push(0 as u8);
            }
        };

        data
    }

    pub fn size(flags: VertexFlag) -> usize {
        flags
            .iter()
            .map(|bit| match bit {
                VertexFlag::POSITION
                | VertexFlag::COLOR
                | VertexFlag::TEXTURE
                | VertexFlag::NORMAL
                | VertexFlag::NORMAL_TEXTURE
                | VertexFlag::TANGENT
                | VertexFlag::BITANGENT => flags.alignment(),
                _ => unimplemented!(),
            })
            .sum()
    }
}

pub struct VertexDataBuilder {
    pub position: Option<Vec3>,
    pub color: Option<Vec4>,
    pub texture: Option<Vec2>,
    pub normal: Option<Vec3>,
    pub normal_texture: Option<Vec2>,
    pub tangent: Option<Vec3>,
    pub bitangent: Option<Vec3>,
    pub index: Option<f32>,
}

impl VertexDataBuilder {
    pub fn new() -> Self {
        VertexDataBuilder {
            position: None,
            color: None,
            texture: None,
            normal: None,
            normal_texture: None,
            tangent: None,
            bitangent: None,
            index: None,
        }
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = Some(position);

        self
    }

    pub fn color(mut self, color: Vec4) -> Self {
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

    pub fn index(mut self, index: f32) -> Self {
        self.index = Some(index);

        self
    }

    pub fn build(self) -> VertexData {
        VertexData {
            position: self.position.unwrap_or(Vec3::splat(0.)),
            color: self.color.unwrap_or(Vec4::splat(1.)),
            texture: self.texture.unwrap_or(Vec2::splat(0.)),
            normal: self.normal.unwrap_or(Vec3::splat(0.)),
            normal_texture: self.normal_texture.unwrap_or(Vec2::splat(0.)),
            tangent: self.tangent.unwrap_or(Vec3::splat(0.)),
            bitangent: self.bitangent.unwrap_or(Vec3::splat(0.)),
            index: self.index.unwrap_or(1.),
        }
    }
}

impl Default for VertexDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
