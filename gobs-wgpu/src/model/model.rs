use crate::model::{Material, Mesh};

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}
