use std::sync::Arc;

use uuid::Uuid;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_uv: [f32; 2],
}

#[derive(Copy, Clone)]
pub enum PrimitiveType {
    Triangle,
    Line
}

pub struct MeshBuilder {
    id: Uuid,
    primitive_type: PrimitiveType,
    vlist: Vec<Vertex>
}

impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder {
            id: Uuid::new_v4(),
            primitive_type: PrimitiveType::Triangle,
            vlist: Vec::new()
        }
    }

    pub fn add_vertex(mut self, position: [f32; 3], normal: [f32; 3], tex_uv: [f32; 2])
    -> Self {
        let vertex = Vertex { position: position, normal: normal, tex_uv: tex_uv };

        self.vlist.push(vertex);

        self
    }

    pub fn line(mut self) -> Self {
        self.primitive_type = PrimitiveType::Line;

        self
    }

    pub fn build(self) -> Arc<Mesh> {
        Mesh::new(self.id, self.primitive_type, self.vlist)
    }
}

pub struct Mesh {
    id: Uuid,
    primitive_type: PrimitiveType,
    vlist: Vec<Vertex>,
}

impl Mesh {
    fn new(id: Uuid, primitive_type: PrimitiveType, vlist: Vec<Vertex>) -> Arc<Mesh> {
        let mesh = Mesh {
            id: id,
            primitive_type: primitive_type,
            vlist: vlist
        };

        Arc::new(mesh)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn vlist(&self) -> &Vec<Vertex> {
        &self.vlist
    }

    pub fn primitive_type(&self) -> PrimitiveType {
            self.primitive_type
    }
}
