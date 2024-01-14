use std::collections::HashMap;

use glam::Vec2;

use crate::geometry::vertex::VertexData;

use super::vertex::VertexFlag;

pub enum PrimitiveType {
    Triangle,
}

pub struct Primitive {
    pub ty: PrimitiveType,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::new()
    }

    pub fn vertices_data(&self, flags: VertexFlag) -> Vec<u8> {
        self.vertices
            .iter()
            .flat_map(|v| v.raw(flags))
            .collect::<Vec<u8>>()
    }
}

pub struct PrimitiveBuilder {
    vertices: Vec<VertexData>,
    indices: Vec<u32>,
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        PrimitiveBuilder {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertices(mut self, vertices: &[VertexData]) -> Self {
        self.vertices.extend_from_slice(vertices);

        self
    }

    pub fn add_vertex(mut self, vertex_data: VertexData) -> Self {
        self.vertices.push(vertex_data);

        self
    }

    pub fn add_indices(mut self, indices: &[u32]) -> Self {
        self.indices.extend(indices);

        self
    }

    fn autoindex(mut self) -> Self {
        if !self.indices.is_empty() {
            log::debug!("Skip indices");
            return self;
        }

        let mut unique = HashMap::new();

        log::debug!("Indexing {} vertices", self.vertices.len());

        let mut idx = 0;
        let vertices = self
            .vertices
            .into_iter()
            .filter(|v| {
                let key = format!("{}:{}:{}", v.position, v.texture, v.normal);
                if unique.contains_key(&key) {
                    let idx = unique.get(&key).unwrap();
                    self.indices.push(*idx);
                    false
                } else {
                    unique.insert(key, idx);
                    self.indices.push(idx);
                    idx += 1;
                    true
                }
            })
            .collect::<Vec<VertexData>>();

        self.vertices = vertices;

        self
    }

    fn update_tangent(mut self) -> Self {
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

    pub fn build(mut self) -> Primitive {
        self = self.autoindex();

        assert_eq!(self.indices.len() % 3, 0);

        self = self.update_tangent();

        log::debug!(
            "Load primitive ({} vertices / {} indices)",
            self.vertices.len(),
            self.indices.len()
        );

        Primitive {
            ty: PrimitiveType::Triangle,
            vertices: self.vertices,
            indices: self.indices,
        }
    }
}
