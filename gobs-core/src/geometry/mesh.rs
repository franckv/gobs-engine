use std::sync::Arc;

use glam::Vec2;
use uuid::Uuid;

use crate::geometry::vertex::{VertexData, VertexFlag};

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Clone, Copy, Debug)]
pub struct Primitive {
    pub ty: PrimitiveType,
    pub offset: usize,
    pub len: usize,
}

pub type MeshId = Uuid;

pub struct Mesh {
    pub id: MeshId,
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn builder(name: &str) -> MeshBuilder {
        MeshBuilder::new(name)
    }

    pub fn vertices_data(&self, flags: VertexFlag) -> Vec<u8> {
        self.vertices
            .iter()
            .flat_map(|v| v.raw(flags))
            .collect::<Vec<u8>>()
    }
}

pub struct MeshBuilder {
    name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub primitives: Vec<Primitive>,
}

impl MeshBuilder {
    fn new(name: &str) -> Self {
        MeshBuilder {
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
            primitives: Vec::new(),
        }
    }

    pub fn vertices(mut self, vertices: Vec<VertexData>) -> Self {
        self.vertices = vertices;

        self
    }

    pub fn indices(mut self, indices: Vec<u32>) -> Self {
        self.indices = indices;

        self
    }

    pub fn add_primitive(mut self, offset: usize, len: usize) -> Self {
        self.primitives.push(Primitive {
            ty: PrimitiveType::Triangle,
            offset,
            len,
        });

        self
    }

    pub fn update_tangent(mut self) -> Self {
        log::debug!("Calculating tangents for {} indices", self.indices.len());

        let mut triangles_included = vec![0; self.vertices.len()];

        for c in self.indices.chunks(3) {
            let v0 = self.vertices[c[0] as usize].clone();
            let v1 = self.vertices[c[1] as usize].clone();
            let v2 = self.vertices[c[2] as usize].clone();

            let pos0 = v0.position;
            let pos1 = v1.position;
            let pos2 = v2.position;

            let uv0: Vec2 = v0.texture;
            let uv1: Vec2 = v1.texture;
            let uv2: Vec2 = v2.texture;

            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1. / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            self.vertices[c[0] as usize].tangent = tangent + v0.tangent;
            self.vertices[c[1] as usize].tangent = tangent + v1.tangent;
            self.vertices[c[2] as usize].tangent = tangent + v2.tangent;
            self.vertices[c[0] as usize].bitangent = bitangent + v0.bitangent;
            self.vertices[c[1] as usize].bitangent = bitangent + v1.bitangent;
            self.vertices[c[2] as usize].bitangent = bitangent + v2.bitangent;

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1. / n as f32;
            let v = &mut self.vertices[i];
            v.tangent = v.tangent * denom;
            v.bitangent = v.bitangent * denom;
        }

        self
    }

    pub fn build(self) -> Arc<Mesh> {
        Arc::new(Mesh {
            id: Uuid::new_v4(),
            name: self.name,
            vertices: self.vertices,
            indices: self.indices,
            primitives: self.primitives,
        })
    }
}
