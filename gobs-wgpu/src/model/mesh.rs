use glam::{Vec2, Vec3};

use crate::render::Gfx;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

impl Mesh {
    pub fn new(
        gfx: &Gfx,
        name: &str,
        vertices: &mut Vec<ModelVertex>,
        indices: &Vec<u32>,
        material: usize,
        calc_tangent: bool,
    ) -> Self {
        if calc_tangent {
            Self::calc_tangent(vertices, indices);
        }

        let vertex_buffer = gfx.create_vertex_buffer(vertices);
        let index_buffer = gfx.create_index_buffer(indices);

        Mesh {
            name: name.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material,
        }
    }

    fn calc_tangent(vertices: &mut Vec<ModelVertex>, indices: &Vec<u32>) {
        let mut triangles_included = vec![0; vertices.len()];

        for c in indices.chunks(3) {
            let v0 = vertices[c[0] as usize];
            let v1 = vertices[c[1] as usize];
            let v2 = vertices[c[2] as usize];

            let pos0: Vec3 = v0.position.into();
            let pos1: Vec3 = v1.position.into();
            let pos2: Vec3 = v2.position.into();

            let uv0: Vec2 = v0.tex_coords.into();
            let uv1: Vec2 = v1.tex_coords.into();
            let uv2: Vec2 = v2.tex_coords.into();

            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            vertices[c[0] as usize].tangent = (tangent + Vec3::from(v0.tangent)).into();
            vertices[c[1] as usize].tangent = (tangent + Vec3::from(v1.tangent)).into();
            vertices[c[2] as usize].tangent = (tangent + Vec3::from(v2.tangent)).into();
            vertices[c[0] as usize].bitangent = (bitangent + Vec3::from(v0.bitangent)).into();
            vertices[c[1] as usize].bitangent = (bitangent + Vec3::from(v1.bitangent)).into();
            vertices[c[2] as usize].bitangent = (bitangent + Vec3::from(v2.bitangent)).into();

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let v = &mut vertices[i];
            v.tangent = (Vec3::from(v.tangent) * denom).into();
            v.bitangent = (Vec3::from(v.bitangent) * denom).into();
        }
    }
}
