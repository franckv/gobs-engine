use std::sync::Arc;

use uuid::Uuid;

use super::primitive::Primitive;

pub type MeshId = Uuid;

pub struct Mesh {
    pub id: MeshId,
    pub name: String,
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn builder(name: &str) -> MeshBuilder {
        MeshBuilder::new(name)
    }
}

pub struct MeshBuilder {
    name: String,
    pub primitives: Vec<Primitive>,
}

impl MeshBuilder {
    pub fn new(name: &str) -> Self {
        MeshBuilder {
            name: name.to_string(),
            primitives: Vec::new(),
        }
    }

    pub fn add_primitive(mut self, primitive: Primitive) -> Self {
        self.primitives.push(primitive);

        self
    }

    pub fn add_primitives(mut self, primitives: &mut Vec<Primitive>) -> Self {
        self.primitives.append(primitives);

        self
    }

    pub fn build(self) -> Arc<Mesh> {
        Arc::new(Mesh {
            id: Uuid::new_v4(),
            name: self.name,
            primitives: self.primitives,
        })
    }
}
