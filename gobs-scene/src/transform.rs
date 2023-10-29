use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Transform {
    pub const IDENTITY: Self = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };

    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        Transform {
            translation: translation,
            rotation: rotation,
        }
    }

    pub fn translation(translation: Vec3) -> Self {
        Self::new(translation, Quat::IDENTITY)
    }

    pub fn rotation(rotation: Quat) -> Self {
        Self::new(Vec3::ZERO, rotation)
    }
}
