use std::{fmt::Debug, ops::Mul};

use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub matrix: Mat4,
}

impl Transform {
    pub const IDENTITY: Self = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        matrix: Mat4::IDENTITY,
    };

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Transform {
            translation,
            rotation,
            scale,
            matrix: Mat4::from_translation(translation)
                * Mat4::from_quat(rotation)
                * Mat4::from_scale(scale),
        }
    }

    pub fn translation(translation: Vec3) -> Self {
        Self::new(translation, Quat::IDENTITY, Vec3::ONE)
    }

    pub fn rotation(rotation: Quat) -> Self {
        Self::new(Vec3::ZERO, rotation, Vec3::ONE)
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.translation = translation + self.translation;
        self.update_matrix();
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
        self.update_matrix();
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.scale = scale * self.scale;
        self.update_matrix();
    }

    fn update_matrix(&mut self) {
        self.matrix = Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale);
    }
}

impl Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = &mut f.debug_struct("Transform");

        if self.translation != Vec3::ZERO {
            builder = builder.field("T", &self.translation);
        }
        if self.rotation != Quat::IDENTITY {
            builder = builder.field("R", &self.rotation);
        }
        if self.scale != Vec3::ONE {
            builder = builder.field("S", &self.scale);
        }

        builder.finish()
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.translation + rhs.translation,
            self.rotation * rhs.rotation,
            self.scale * rhs.scale,
        )
    }
}

impl Into<Mat4> for Transform {
    fn into(self) -> Mat4 {
        self.matrix
    }
}

impl Into<[[f32; 4]; 4]> for Transform {
    fn into(self) -> [[f32; 4]; 4] {
        self.matrix.to_cols_array_2d()
    }
}
