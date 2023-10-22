use uuid::Uuid;

use crate::model::{Material, Mesh};

pub struct ModelBuilder {
    id: Uuid,
    scale: f32,
    meshes: Vec<Mesh>,
    materials: Vec<Material>
}

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder {
            id: Uuid::new_v4(),
            scale: 1.,
            meshes: Vec::new(),
            materials: Vec::new()
        }
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;

        self
    }

    pub fn add_mesh(mut self, mut mesh: Mesh, material: usize) -> Self {
        mesh.material = material;
        self.meshes.push(mesh);

        self
    }

    pub fn add_material(mut self, material: Material) -> Self {
        self.materials.push(material);

        self
    }
    
    pub fn build(self) -> Model {
        Model {
            id: self.id,
            scale: self.scale,
            meshes: self.meshes,
            materials: self.materials
        }
    }
}

pub struct Model {
    pub id: Uuid,
    pub scale: f32,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}
