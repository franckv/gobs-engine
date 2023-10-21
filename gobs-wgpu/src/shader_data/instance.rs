use bitflags::bitflags;
use glam::{Mat3, Mat4, Quat, Vec3};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InstanceFlag: u32 {
        const MODEL = 1;
        const NORMAL = 1 << 1;

        const MN = InstanceFlag::MODEL.bits() | InstanceFlag::NORMAL.bits();
    }
}

pub enum InstanceData {
    InstanceMN(InstanceMN),
    InstanceM(InstanceM),
}

impl InstanceData {
    pub fn new(
        flags: InstanceFlag,
        position: Vec3,
        rotation: Quat,
        scale: f32,
    ) -> Result<Self, ()> {
        match flags {
            InstanceFlag::MODEL => Ok(InstanceData::InstanceM(InstanceM::new(
                position, rotation, scale,
            ))),
            InstanceFlag::MN => Ok(InstanceData::InstanceMN(InstanceMN::new(
                position, rotation, scale,
            ))),
            _ => Err(()),
        }
    }

    pub fn size(flags: InstanceFlag) -> usize {
        match flags {
            InstanceFlag::MODEL => std::mem::size_of::<InstanceM>(),
            InstanceFlag::MN => std::mem::size_of::<InstanceMN>(),
            _ => 0,
        }
    }

    pub fn raw(&self) -> &[u8] {
        match self {
            InstanceData::InstanceMN(data) => bytemuck::bytes_of(data),
            InstanceData::InstanceM(data) => bytemuck::bytes_of(data),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceMN {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl InstanceMN {
    fn new(position: Vec3, rotation: Quat, scale: f32) -> Self {
        InstanceMN {
            model: (Mat4::from_translation(position)
                * Mat4::from_quat(rotation)
                * Mat4::from_scale(Vec3::splat(scale)))
            .to_cols_array_2d(),
            normal: Mat3::from_quat(rotation).to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceM {
    model: [[f32; 4]; 4],
}

impl InstanceM {
    fn new(position: Vec3, rotation: Quat, scale: f32) -> Self {
        InstanceM {
            model: (Mat4::from_translation(position)
                * Mat4::from_quat(rotation)
                * Mat4::from_scale(Vec3::splat(scale)))
            .to_cols_array_2d(),
        }
    }
}
