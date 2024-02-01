use std::sync::Arc;

use glam::Vec2;

use gobs_material::vertex::VertexData;

pub struct Mesh {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl Mesh {
    fn new(vertices: Vec<VertexData>, indices: Vec<u32>) -> Arc<Self> {
        Arc::new(Self { vertices, indices })
    }

    pub fn builder() -> MeshBuilder {
        MeshBuilder::new()
    }

    pub fn update_tangent(vertices: &mut [VertexData], indices: &[u32]) {
        log::debug!("Calculating tangents for {} indices", indices.len());

        let mut triangles_included = vec![0; vertices.len()];

        for c in indices.chunks(3) {
            let v0 = vertices[c[0] as usize].clone();
            let v1 = vertices[c[1] as usize].clone();
            let v2 = vertices[c[2] as usize].clone();

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

            vertices[c[0] as usize].tangent = tangent + v0.tangent;
            vertices[c[1] as usize].tangent = tangent + v1.tangent;
            vertices[c[2] as usize].tangent = tangent + v2.tangent;
            vertices[c[0] as usize].bitangent = bitangent + v0.bitangent;
            vertices[c[1] as usize].bitangent = bitangent + v1.bitangent;
            vertices[c[2] as usize].bitangent = bitangent + v2.bitangent;

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1. / n as f32;
            let v = &mut vertices[i];
            v.tangent = v.tangent * denom;
            v.bitangent = v.bitangent * denom;
        }
    }
}

pub struct MeshBuilder {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex(mut self, data: VertexData) -> Self {
        self.vertices.push(data);

        self
    }

    pub fn index(mut self, idx: u32) -> Self {
        self.indices.push(idx);

        self
    }

    pub fn build(self) -> Arc<Mesh> {
        Mesh::new(self.vertices, self.indices)
    }
}
