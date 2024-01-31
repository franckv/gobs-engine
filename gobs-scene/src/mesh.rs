use std::sync::Arc;

use glam::Vec2;
use uuid::Uuid;

use gobs_material::vertex::{VertexData, VertexFlag};
use gobs_render::context::Context;
use gobs_vulkan::buffer::{Buffer, BufferUsage};

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Clone, Copy, Debug)]
pub struct Primitive {
    pub ty: PrimitiveType,
    pub offset: usize,
    pub len: usize,
    pub material: usize,
}

impl Primitive {
    pub fn new(ty: PrimitiveType, offset: usize, len: usize, material: usize) -> Self {
        Primitive {
            ty,
            offset,
            len,
            material,
        }
    }
}

pub type MeshId = Uuid;

pub struct Mesh {
    pub name: String,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn new(
        ctx: &Context,
        name: &str,
        vertices: &[VertexData],
        indices: &[u32],
        primitives: Vec<Primitive>,
        vertex_flags: VertexFlag,
    ) -> Arc<Self> {
        let vertices_data = vertices
            .iter()
            .flat_map(|v| v.raw(vertex_flags))
            .collect::<Vec<u8>>();
        let vertices_size = vertices_data.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();

        let mut staging = Buffer::new(
            "staging",
            indices_size + vertices_size,
            BufferUsage::Staging,
            ctx.device.clone(),
            ctx.allocator.clone(),
        );

        let index_buffer = Buffer::new(
            "index",
            indices_size,
            BufferUsage::Index,
            ctx.device.clone(),
            ctx.allocator.clone(),
        );
        let vertex_buffer = Buffer::new(
            "vertex",
            vertices_size,
            BufferUsage::Vertex,
            ctx.device.clone(),
            ctx.allocator.clone(),
        );

        staging.copy(&vertices_data, 0);
        staging.copy(indices, vertices_size);

        ctx.immediate_cmd.immediate(|cmd| {
            cmd.begin_label("Upload buffer");

            cmd.copy_buffer(&staging, &vertex_buffer, vertex_buffer.size, 0);
            cmd.copy_buffer(
                &staging,
                &index_buffer,
                index_buffer.size,
                vertex_buffer.size,
            );
            cmd.end_label();
        });

        Arc::new(Mesh {
            name: name.to_string(),
            index_buffer,
            vertex_buffer,
            primitives,
        })
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
