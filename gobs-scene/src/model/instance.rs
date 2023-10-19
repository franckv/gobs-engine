use glam::{Quat, Vec3};

#[derive(Clone)]
pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
}
