use bitflags::bitflags;
use glam::{Vec2, Vec3};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VertexFlag: u32 {
        const POSITION = 1;
        const TEXTURE = 1 << 1;
        const NORMAL = 1 << 2;

        const PT = VertexFlag::POSITION.bits() | VertexFlag::TEXTURE.bits();
        const PTN = VertexFlag::POSITION.bits() | VertexFlag::TEXTURE.bits() | VertexFlag::NORMAL.bits();
    }
}

#[derive(Clone, Copy)]
pub enum VertexData {
    VertexP(VertexP),
    VertexPT(VertexPT),
    VertexPTN(VertexPTN),
}

impl VertexData {
    pub fn new(flags: VertexFlag, position: Vec3) -> Result<Self, ()> {
        match flags {
            VertexFlag::POSITION => Ok(VertexData::VertexP(VertexP::new(position))),
            _ => Err(()),
        }
    }

    pub fn size(flags: VertexFlag) -> usize {
        match flags {
            VertexFlag::POSITION => std::mem::size_of::<VertexP>(),
            VertexFlag::PT => std::mem::size_of::<VertexPT>(),
            VertexFlag::PTN => std::mem::size_of::<VertexPTN>(),
            _ => 0,
        }
    }

    pub fn raw(&self) -> &[u8] {
        match self {
            VertexData::VertexP(data) => bytemuck::bytes_of(data),
            VertexData::VertexPT(data) => bytemuck::bytes_of(data),
            VertexData::VertexPTN(data) => bytemuck::bytes_of(data),
        }
    }

    pub fn position(&self) -> Vec3 {
        match self {
            VertexData::VertexP(data) => data.position.into(),
            VertexData::VertexPT(data) => data.position.into(),
            VertexData::VertexPTN(data) => data.position.into(),
        }
    }

    pub fn tex_coords(&self) -> Vec2 {
        match self {
            VertexData::VertexP(_) => Vec2::splat(0.),
            VertexData::VertexPT(data) => data.tex_coords.into(),
            VertexData::VertexPTN(data) => data.tex_coords.into(),
        }
    }

    pub fn normal(&self) -> Vec3 {
        match self {
            VertexData::VertexP(_) => Vec3::splat(0.),
            VertexData::VertexPT(_) => Vec3::splat(0.),
            VertexData::VertexPTN(data) => data.normal.into(),
        }
    }

    pub fn tangent(&self) -> Vec3 {
        match self {
            VertexData::VertexP(_) => Vec3::splat(0.),
            VertexData::VertexPT(_) => Vec3::splat(0.),
            VertexData::VertexPTN(data) => data.tangent.into(),
        }
    }

    pub fn set_tangent(&mut self, tangent: Vec3) {
        match self {
            VertexData::VertexP(_) => (),
            VertexData::VertexPT(_) => (),
            VertexData::VertexPTN(data) => data.tangent = tangent.into(),
        }
    }

    pub fn bitangent(&self) -> Vec3 {
        match self {
            VertexData::VertexP(_) => Vec3::splat(0.),
            VertexData::VertexPT(_) => Vec3::splat(0.),
            VertexData::VertexPTN(data) => data.bitangent.into(),
        }
    }

    pub fn set_bitangent(&mut self, bitangent: Vec3) {
        match self {
            VertexData::VertexP(_) => (),
            VertexData::VertexPT(_) => (),
            VertexData::VertexPTN(data) => data.bitangent = bitangent.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexP {
    pub position: [f32; 3],
}

impl VertexP {
    fn new(position: Vec3) -> Self {
        VertexP {
            position: position.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPT {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPTN {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}
