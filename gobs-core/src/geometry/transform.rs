use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub matrix: Mat4,
}

impl Transform {
    pub const IDENTITY: Self = Transform {
        matrix: Mat4::IDENTITY,
    };

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Transform {
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
