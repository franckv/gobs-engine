use std::{
    collections::{HashMap, hash_map},
    sync::Arc,
};

use glam::{Vec2, Vec3};
use gobs_core::Transform;
use serde::Serialize;
use uuid::Uuid;

use crate::geometry::{Bounded, BoundingBox, VertexData};

pub type MeshId = Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct MeshGeometry {
    pub id: MeshId,
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl MeshGeometry {
    fn new(name: String, vertices: Vec<VertexData>, indices: Vec<u32>) -> Arc<Self> {
        Arc::new(Self {
            id: Uuid::new_v4(),
            name,
            vertices,
            indices,
        })
    }

    pub fn builder(name: &str) -> MeshBuilder {
        MeshBuilder::new(name)
    }
}

impl Bounded for MeshGeometry {
    fn boundings(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        for vertex in &self.vertices {
            bounding_box.extends(vertex.position);
        }

        bounding_box
    }
}

pub struct MeshBuilder {
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub generate_tangents: bool,
}

impl MeshBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
            generate_tangents: true,
        }
    }

    pub fn vertex(mut self, data: VertexData) -> Self {
        self.vertices.push(data);

        self
    }

    pub fn vertices(mut self, data: &[VertexData]) -> Self {
        self.vertices.extend(data);

        self
    }

    pub fn vertices_with_transform(mut self, data: &[VertexData], transform: Transform) -> Self {
        self.vertices
            .extend(data.iter().map(|v| v.transform(transform)));

        self
    }

    pub fn index(mut self, idx: u32) -> Self {
        self.indices.push(idx);

        self
    }

    pub fn indices(mut self, indices: &[u32], append: bool) -> Self {
        if append {
            let start = self.vertices.len();

            self.indices
                .extend(indices.iter().map(|&i| i + start as u32));
        } else {
            self.indices.extend(indices);
        }

        self
    }

    pub fn generate_tangents(mut self, generate_tangents: bool) -> Self {
        self.generate_tangents = generate_tangents;

        self
    }

    pub fn extend(mut self, mesh: Arc<MeshGeometry>) -> Self {
        self = self.indices(&mesh.indices, true).vertices(&mesh.vertices);

        self
    }

    fn autoindex(mut self) -> Self {
        if !self.indices.is_empty() {
            tracing::trace!(target: "resources", "Skip indices");
            return self;
        }

        let mut unique = HashMap::new();

        tracing::trace!(target: "resources", "Indexing {} vertices", self.vertices.len());

        let mut idx = 0;
        let vertices = self
            .vertices
            .into_iter()
            .filter(|v| {
                let key = format!("{}:{}:{}", v.position, v.texture, v.normal);
                if let hash_map::Entry::Vacant(e) = unique.entry(key.clone()) {
                    e.insert(idx);
                    self.indices.push(idx);
                    idx += 1;
                    true
                } else {
                    let idx = unique.get(&key).unwrap();
                    self.indices.push(*idx);
                    false
                }
            })
            .collect::<Vec<VertexData>>();

        self.vertices = vertices;

        self
    }

    fn get_tangents(v0: &VertexData, v1: &VertexData, v2: &VertexData) -> (Vec3, Vec3) {
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

        let d = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;
        if d == 0. {
            let normal = (v0.normal + v1.normal + v2.normal).normalize();
            let tangent = delta_pos1.normalize();
            let bitangent = normal.cross(tangent);

            (tangent, bitangent)
        } else {
            let r = 1. / d;
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            (tangent, bitangent)
        }
    }

    fn update_tangent(mut self) -> Self {
        tracing::trace!(target: "resources", "Calculating tangents for {} indices", self.indices.len());

        let mut triangles_included = vec![0; self.vertices.len()];

        for c in self.indices.chunks(3) {
            let v0 = &mut self.vertices[c[0] as usize].clone();
            let v1 = &mut self.vertices[c[1] as usize].clone();
            let v2 = &mut self.vertices[c[2] as usize].clone();

            let (tangent, bitangent) = Self::get_tangents(v0, v1, v2);

            self.vertices[c[0] as usize].tangent += tangent;
            self.vertices[c[1] as usize].tangent += tangent;
            self.vertices[c[2] as usize].tangent += tangent;
            self.vertices[c[0] as usize].bitangent += bitangent;
            self.vertices[c[1] as usize].bitangent += bitangent;
            self.vertices[c[2] as usize].bitangent += bitangent;

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1. / n as f32;
            let v = &mut self.vertices[i];
            v.tangent *= denom;
            v.bitangent *= denom;
        }

        self
    }

    pub fn build(mut self) -> Arc<MeshGeometry> {
        self = self.autoindex();

        assert_eq!(self.indices.len() % 3, 0);

        if self.generate_tangents {
            self = self.update_tangent();
        }

        tracing::debug!(target: "resources",
            "Load mesh {} ({} vertices / {} indices)",
            self.name,
            self.vertices.len(),
            self.indices.len()
        );

        MeshGeometry::new(self.name, self.vertices, self.indices)
    }
}
