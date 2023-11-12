use bitflags::bitflags;
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InstanceFlag: u32 {
        const MODEL = 1;
        const NORMAL = 1 << 1;
        const TEXTURE = 1 << 2;
    }
}

pub struct InstanceDataBuilder {
    model: Option<Mat4>,
    normal: Option<Mat3>,
    texture_map: Option<Vec4>,
}

impl InstanceDataBuilder {
    pub fn model(mut self, model: Mat4) -> Self {
        self.model = Some(model);

        self
    }

    pub fn model_transform(self, position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        self.model(
            Mat4::from_translation(position) * Mat4::from_quat(rotation) * Mat4::from_scale(scale),
        )
    }

    pub fn normal(mut self, normal: Mat3) -> Self {
        self.normal = Some(normal);

        self
    }

    pub fn normal_rot(self, rotation: Quat) -> Self {
        self.normal(Mat3::from_quat(rotation))
    }

    pub fn texture_map(mut self, texture_map: Vec4) -> Self {
        self.texture_map = Some(texture_map);

        self
    }

    pub fn build(self) -> InstanceData {
        InstanceData {
            model: self.model.unwrap_or(Mat4::IDENTITY),
            normal: self.normal.unwrap_or(Mat3::IDENTITY),
            texture_map: self.texture_map.unwrap_or(Vec4::new(0., 1., 0., 1.)),
        }
    }
}

pub struct InstanceData {
    model: Mat4,
    normal: Mat3,
    texture_map: Vec4,
}

impl InstanceData {
    pub fn new() -> InstanceDataBuilder {
        InstanceDataBuilder {
            model: None,
            normal: None,
            texture_map: None,
        }
    }

    pub fn raw(&self, flags: InstanceFlag) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        if flags.contains(InstanceFlag::MODEL) {
            data.extend_from_slice(bytemuck::cast_slice(&self.model.to_cols_array()));
        };

        if flags.contains(InstanceFlag::NORMAL) {
            data.extend_from_slice(bytemuck::cast_slice(&self.normal.to_cols_array()));
        };

        if flags.contains(InstanceFlag::TEXTURE) {
            data.extend_from_slice(bytemuck::cast_slice(&self.texture_map.to_array()));
        };

        data
    }

    pub fn size(flags: InstanceFlag) -> usize {
        flags
            .iter()
            .map(|bit| match bit {
                InstanceFlag::MODEL => std::mem::size_of::<Mat4>(),
                InstanceFlag::NORMAL => std::mem::size_of::<Mat3>(),
                InstanceFlag::TEXTURE => std::mem::size_of::<Vec4>(),
                _ => unimplemented!(),
            })
            .sum()
    }
}
