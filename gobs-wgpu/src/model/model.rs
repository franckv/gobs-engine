use uuid::Uuid;

use crate::model::{Material, Mesh};

pub struct Model {
    pub id: Uuid,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}
