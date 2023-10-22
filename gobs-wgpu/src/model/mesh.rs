use std::collections::HashMap;

use glam::{Vec2, Vec3};
use log::{error, info};
use uuid::Uuid;

use crate::render::Gfx;
use crate::shader_data::{VertexData, VertexFlag, VertexP, VertexPTN};

pub struct MeshBuilder {
    id: Uuid,
    name: String,
    vertices: Vec<VertexData>,
    indices: Vec<u32>,
    material: usize,
    flags: VertexFlag,
}

#[allow(non_snake_case)]
impl MeshBuilder {
    pub fn new(name: &str, flags: VertexFlag) -> Self {
        MeshBuilder {
            id: Uuid::new_v4(),
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
            material: 0,
            flags,
        }
    }

    pub fn add_vertex_P(mut self, position: Vec3) -> Self {
        assert!(self.flags == VertexFlag::POSITION);

        let vertex = VertexData::VertexP(VertexP {
            position: position.into(),
        });

        self.vertices.push(vertex);

        self
    }

    pub fn add_vertex_PTN(mut self, position: Vec3, texture: Vec2, normal: Vec3) -> Self {
        assert!(self.flags == VertexFlag::PTN);
        
        let vertex = VertexData::VertexPTN(VertexPTN {
            position: position.into(),
            tex_coords: texture.into(),
            normal: normal.into(),
            tangent: Vec3::splat(0.).into(),
            bitangent: Vec3::splat(0.).into(),
        });

        self.vertices.push(vertex);

        self
    }

    pub fn add_indices(mut self, indices: &Vec<u32>) -> Self {
        self.indices.extend(indices);

        self
    }

    pub fn material(mut self, material: usize) -> Self {
        self.material = material;

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
                let key = format!("{}:{}:{}", v.position(), v.tex_coords(), v.normal());
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

        if self.indices.len() % 3 != 0 {
            error!("Vertices number shoud be multiple of 3");
            return self;
        }

        let mut triangles_included = vec![0; self.vertices.len()];

        for c in self.indices.chunks(3) {
            let v0 = self.vertices[c[0] as usize];
            let v1 = self.vertices[c[1] as usize];
            let v2 = self.vertices[c[2] as usize];

            let pos0: Vec3 = v0.position();
            let pos1: Vec3 = v1.position();
            let pos2: Vec3 = v2.position();

            let uv0: Vec2 = v0.tex_coords();
            let uv1: Vec2 = v1.tex_coords();
            let uv2: Vec2 = v2.tex_coords();

            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
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
            let denom = 1.0 / n as f32;
            let v = &mut self.vertices[i];
            v.set_tangent(Vec3::from(v.tangent()) * denom);
            v.set_bitangent(Vec3::from(v.bitangent()) * denom);
        }

        self
    }

    pub fn build(mut self, gfx: &Gfx) -> Mesh {
        self = self.autoindex();
        self = self.update_tangent();

        let vertex_buffer = gfx.create_vertex_buffer(&self.vertices);
        let index_buffer = gfx.create_index_buffer(&self.indices);
        let num_elements = self.indices.len();

        info!(
            "Load mesh {} ({} vertices / {} indices)",
            self.name,
            self.vertices.len(),
            self.indices.len()
        );

        Mesh {
            id: self.id,
            name: self.name,
            vertex_buffer,
            index_buffer,
            num_elements,
            material: self.material,
        }
    }
}

pub struct Mesh {
    pub id: Uuid,
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: usize,
    pub material: usize,
}
