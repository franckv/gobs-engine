use std::{
    collections::{HashMap, hash_map},
    sync::Arc,
};

use glam::{Vec2, Vec3};
use serde::Serialize;
use uuid::Uuid;

use gobs_core::{Transform, logger};

use crate::{
    VertexData,
    mesh::{Bounded, BoundingBox},
};

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
            id: MeshId::new_v4(),
            name,
            vertices,
            indices,
        })
    }

    pub fn builder(name: &str) -> MeshBuilder {
        MeshBuilder::new(name, 0, 0)
    }

    pub fn builder_with_capacity(name: &str, v_capacity: usize, i_capacity: usize) -> MeshBuilder {
        MeshBuilder::new(name, v_capacity, i_capacity)
    }
}

impl Bounded for MeshGeometry {
    fn boundings(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        for vertex in &self.vertices {
            bounding_box.extends(vertex.position());
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
    fn new(name: &str, v_capacity: usize, i_capacity: usize) -> Self {
        let vertices = if v_capacity > 0 {
            Vec::with_capacity(v_capacity)
        } else {
            Vec::new()
        };

        let indices = if i_capacity > 0 {
            Vec::with_capacity(i_capacity)
        } else {
            Vec::new()
        };

        Self {
            name: name.to_string(),
            vertices,
            indices,
            generate_tangents: true,
        }
    }

    pub fn vertex(&mut self, data: VertexData) -> &mut Self {
        self.vertices.push(data);

        self
    }

    pub fn vertices(&mut self, data: &[VertexData]) -> &mut Self {
        self.vertices.extend_from_slice(data);

        self
    }

    pub fn vertices_with_transform(
        &mut self,
        data: &[VertexData],
        transform: Transform,
    ) -> &mut Self {
        self.vertices
            .extend(data.iter().map(|v| v.transform(transform)));

        self
    }

    pub fn index(&mut self, idx: u32) -> &mut Self {
        self.indices.push(idx);

        self
    }

    pub fn indices(&mut self, indices: &[u32], append: bool) -> &mut Self {
        if append {
            let start = self.vertices.len();

            self.indices
                .extend(indices.iter().map(|&i| i + start as u32));
        } else {
            self.indices.extend(indices);
        }

        self
    }

    pub fn generate_tangents(&mut self, generate_tangents: bool) -> &mut Self {
        self.generate_tangents = generate_tangents;

        self
    }

    pub fn extend(&mut self, mesh: Arc<MeshGeometry>) -> &mut Self {
        self.indices(&mesh.indices, true).vertices(&mesh.vertices);

        self
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn autoindex(&mut self) -> &mut Self {
        if !self.indices.is_empty() {
            tracing::trace!(target: logger::RESOURCES, "Skip indices");
            return self;
        }

        tracing::trace!(target: logger::RESOURCES, "Indexing {} vertices", self.vertices.len());

        let mut unique = HashMap::with_capacity(self.vertices.len());
        let mut idx = 0;

        let vertices = std::mem::take(&mut self.vertices)
            .into_iter()
            .filter(|v| {
                let (pos, tex, norm) = (v.position(), v.texture(), v.normal());

                let key = [
                    pos.x.to_bits(),
                    pos.y.to_bits(),
                    pos.z.to_bits(),
                    tex.x.to_bits(),
                    tex.y.to_bits(),
                    norm.x.to_bits(),
                    norm.y.to_bits(),
                    norm.z.to_bits(),
                ];

                if let hash_map::Entry::Vacant(e) = unique.entry(key) {
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

        tracing::debug!(target: logger::RESOURCES, "Autoindex {} vertices, {} indices", self.vertices.len(), self.indices.len());

        self
    }

    fn get_tangents(v0: &VertexData, v1: &VertexData, v2: &VertexData) -> (Vec3, Vec3) {
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

        let d = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;
        if d == 0. {
            let normal = (v0.normal() + v1.normal() + v2.normal()).normalize();
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn update_tangent(&mut self) -> &mut Self {
        tracing::trace!(target: logger::RESOURCES, "Calculating tangents for {} indices", self.indices.len());

        let mut triangles_included = vec![0; self.vertices.len()];

        for c in self.indices.chunks(3) {
            let (tangent, bitangent) = Self::get_tangents(
                &self.vertices[c[0] as usize],
                &self.vertices[c[1] as usize],
                &self.vertices[c[2] as usize],
            );

            let (t0, b0) = (
                self.vertices[c[0] as usize].tangent(),
                self.vertices[c[0] as usize].bitangent(),
            );
            let (t1, b1) = (
                self.vertices[c[1] as usize].tangent(),
                self.vertices[c[1] as usize].bitangent(),
            );
            let (t2, b2) = (
                self.vertices[c[2] as usize].tangent(),
                self.vertices[c[2] as usize].bitangent(),
            );

            self.vertices[c[0] as usize].set_tangent(t0 + tangent);
            self.vertices[c[1] as usize].set_tangent(t1 + tangent);
            self.vertices[c[2] as usize].set_tangent(t2 + tangent);
            self.vertices[c[0] as usize].set_bitangent(b0 + bitangent);
            self.vertices[c[1] as usize].set_bitangent(b1 + bitangent);
            self.vertices[c[2] as usize].set_bitangent(b2 + bitangent);

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1. / n as f32;
            let v = &mut self.vertices[i];
            v.set_tangent(v.tangent() * denom);
            v.set_bitangent(v.bitangent() * denom);
        }

        self
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn build(mut self) -> Arc<MeshGeometry> {
        self.autoindex();

        assert_eq!(self.indices.len() % 3, 0);

        if self.generate_tangents {
            self.update_tangent();
        }

        tracing::debug!(target: logger::RESOURCES,
            "Load mesh {} ({} vertices / {} indices)",
            self.name,
            self.vertices.len(),
            self.indices.len()
        );

        MeshGeometry::new(self.name, self.vertices, self.indices)
    }
}
