use glam::{Mat3, Mat4, Quat, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl InstanceRaw {
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        InstanceRaw {
            model: (Mat4::from_translation(position) * Mat4::from_quat(rotation))
                .to_cols_array_2d(),
            normal: Mat3::from_quat(rotation).to_cols_array_2d(),
        }
    }
}
