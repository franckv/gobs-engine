#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub(crate) view_position: [f32; 4],
    pub(crate) view_proj: [[f32; 4]; 4],
}
