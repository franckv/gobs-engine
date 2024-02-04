use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy, Debug)]
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

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
        self.update_matrix();
    }

    fn update_matrix(&mut self) {
        self.matrix = Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale);
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
