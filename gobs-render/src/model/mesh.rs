use std::collections::HashMap;
use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4};
use log::info;
use uuid::Uuid;

use crate::model::VertexData;

pub struct MeshBuilder {
    name: String,
    vertices: Vec<VertexData>,
    indices: Vec<u32>,
}

#[allow(non_snake_case)]
impl MeshBuilder {
    pub fn new(name: &str) -> Self {
        MeshBuilder {
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(
        mut self,
        position: Vec3,
        color: Vec4,
        texture: Vec2,
        normal: Vec3,
        normal_texture: Vec2,
    ) -> Self {
        let vertex = VertexData::new()
            .position(position)
            .color(color)
            .texture(texture)
            .normal_texture(normal_texture)
            .normal(normal)
            .tangent(Vec3::splat(0.))
            .bitangent(Vec3::splat(0.))
            .build();

        self.vertices.push(vertex);

        self
    }

    pub fn add_indices(mut self, indices: &Vec<u32>) -> Self {
        self.indices.extend(indices);

        self
    }

    fn autoindex(mut self) -> Self {
        if self.indices.len() > 0 {
            info!("Skip indices");
            return self;
        }

        let mut unique = HashMap::new();

        info!("Indexing {} vertices", self.vertices.len());

        let mut idx = 0;
        let vertices = self
            .vertices
            .into_iter()
            .filter(|v| {
                let key = format!("{}:{}:{}", v.position(), v.texture(), v.normal());
                if unique.contains_key(&key) {
                    let idx = unique.get(&key).unwrap();
                    self.indices.push(*idx);
                    return false;
                } else {
                    unique.insert(key, idx);
                    self.indices.push(idx);
                    idx += 1;
                    return true;
                }
            })
            .collect::<Vec<VertexData>>();

        info!(
            "Load {} vertices {} indices",
            vertices.len(),
            self.indices.len()
        );

        self.vertices = vertices;

        self
    }

    fn update_tangent(mut self) -> Self {
        info!("Calculating tangents for {} indices", self.indices.len());

        let mut triangles_included = vec![0; self.vertices.len()];

        for c in self.indices.chunks(3) {
            let v0 = self.vertices[c[0] as usize].clone();
            let v1 = self.vertices[c[1] as usize].clone();
            let v2 = self.vertices[c[2] as usize].clone();

            let pos0 = v0.position();
            let pos1 = v1.position();
            let pos2 = v2.position();

            let uv0: Vec2 = v0.texture();
            let uv1: Vec2 = v1.texture();
            let uv2: Vec2 = v2.texture();

            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1. / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            self.vertices[c[0] as usize].set_tangent(tangent + Vec3::from(v0.tangent()));
            self.vertices[c[1] as usize].set_tangent(tangent + Vec3::from(v1.tangent()));
            self.vertices[c[2] as usize].set_tangent(tangent + Vec3::from(v2.tangent()));
            self.vertices[c[0] as usize].set_bitangent(bitangent + Vec3::from(v0.bitangent()));
            self.vertices[c[1] as usize].set_bitangent(bitangent + Vec3::from(v1.bitangent()));
            self.vertices[c[2] as usize].set_bitangent(bitangent + Vec3::from(v2.bitangent()));

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1. / n as f32;
            let v = &mut self.vertices[i];
            v.set_tangent(Vec3::from(v.tangent()) * denom);
            v.set_bitangent(Vec3::from(v.bitangent()) * denom);
        }

        self
    }

    pub fn build(mut self) -> Arc<Mesh> {
        self = self.autoindex();

        assert_eq!(self.indices.len() % 3, 0);

        self = self.update_tangent();

        info!(
            "Load mesh {} ({} vertices / {} indices)",
            self.name,
            self.vertices.len(),
            self.indices.len()
        );

        Arc::new(Mesh {
            id: Uuid::new_v4(),
            name: self.name,
            vertices: self.vertices,
            indices: self.indices,
        })
    }
}

pub type MeshId = Uuid;

pub struct Mesh {
    pub id: Uuid,
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}
