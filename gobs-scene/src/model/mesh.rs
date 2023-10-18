use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_uv: [f32; 2],
}

#[derive(Copy, Clone)]
pub enum PrimitiveType {
    Triangle,
    Line,
}

pub struct MeshBuilder {
    id: Uuid,
    primitive_type: PrimitiveType,
    vlist: Vec<Vertex>,
    ilist: Vec<u32>,
    autoindex: bool,
}

impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder {
            id: Uuid::new_v4(),
            primitive_type: PrimitiveType::Triangle,
            vlist: Vec::new(),
            ilist: Vec::new(),
            autoindex: false,
        }
    }

    pub fn add_vertex(mut self, position: [f32; 3], normal: [f32; 3], tex_uv: [f32; 2]) -> Self {
        let vertex = Vertex {
            position,
            normal,
            tex_uv,
        };

        self.vlist.push(vertex);

        self
    }

    pub fn add_indice(mut self, indice: u32) {
        self.ilist.push(indice)
    }

    pub fn line(mut self) -> Self {
        self.primitive_type = PrimitiveType::Line;

        self
    }

    pub fn autoindex(mut self) -> Self {
        self.autoindex = true;

        self
    }

    pub fn build(self) -> Arc<Mesh> {
        if self.autoindex && self.ilist.len() == 0 {
            let mut ilist = Vec::new();
            let mut vlist = Vec::new();

            let mut unique = HashMap::new();
            let mut idx = 0;
            for v in &self.vlist {
                let key = format!(
                    "{}:{}:{}:{}:{}:{}:{}:{}",
                    v.position[0],
                    v.position[1],
                    v.position[2],
                    v.normal[0],
                    v.normal[1],
                    v.normal[2],
                    v.tex_uv[0],
                    v.tex_uv[1]
                );

                if unique.contains_key(&key) {
                    let dup_idx = unique.get(&key).unwrap();
                    ilist.push(*dup_idx);
                } else {
                    unique.insert(key, idx);
                    ilist.push(idx);
                    vlist.push(v.clone());
                    idx += 1;
                }
            }

            Mesh::new(self.id, self.primitive_type, vlist, Some(ilist))
        } else if self.ilist.len() > 0 {
            Mesh::new(self.id, self.primitive_type, self.vlist, Some(self.ilist))
        } else {
            Mesh::new(self.id, self.primitive_type, self.vlist, None)
        }
    }
}

pub struct Mesh {
    id: Uuid,
    primitive_type: PrimitiveType,
    vlist: Vec<Vertex>,
    ilist: Option<Vec<u32>>,
}

impl Mesh {
    fn new(
        id: Uuid,
        primitive_type: PrimitiveType,
        vlist: Vec<Vertex>,
        ilist: Option<Vec<u32>>,
    ) -> Arc<Mesh> {
        let mesh = Mesh {
            id,
            primitive_type,
            vlist,
            ilist,
        };

        Arc::new(mesh)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn vlist(&self) -> &Vec<Vertex> {
        &self.vlist
    }

    pub fn ilist(&self) -> &Option<Vec<u32>> {
        &self.ilist
    }

    pub fn primitive_type(&self) -> PrimitiveType {
        self.primitive_type
    }
}
