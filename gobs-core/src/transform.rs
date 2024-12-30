use std::{fmt::Debug, ops::Mul};

use glam::{Mat4, Quat, Vec3, Vec4};

#[derive(Clone, Copy)]
pub struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
    matrix: Mat4,
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

    pub fn from_translation(translation: Vec3) -> Self {
        Self::new(translation, Quat::IDENTITY, Vec3::ONE)
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self::new(Vec3::ZERO, rotation, Vec3::ONE)
    }

    pub fn matrix(&self) -> &Mat4 {
        &self.matrix
    }

    pub fn translation(&self) -> Vec3 {
        self.translation
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn scaling(&self) -> Vec3 {
        self.scale
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;
        self.update_matrix();
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
        let matrix = self.matrix * rhs.matrix;
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Self::new(translation, rotation, scale)
    }
}

impl Mul<Vec4> for Transform {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        self.matrix * rhs
    }
}

impl From<Transform> for Mat4 {
    fn from(val: Transform) -> Self {
        val.matrix
    }
}

impl From<Transform> for [[f32; 4]; 4] {
    fn from(val: Transform) -> Self {
        val.matrix.to_cols_array_2d()
    }
}
