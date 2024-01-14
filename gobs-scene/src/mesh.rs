use std::sync::{Arc, Mutex};

use gpu_allocator::vulkan::Allocator;

use gobs_core::geometry::{
    mesh::Mesh,
    vertex::{VertexData, VertexFlag},
};
use gobs_render::context::Context;
use gobs_vulkan::buffer::{Buffer, BufferUsage};

pub struct MeshSurface {
    pub offset: usize,
    pub len: usize,
}

pub struct MeshBuffer {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
}

impl MeshBuffer {
    pub fn new(
        ctx: &Context,
        mesh: Arc<Mesh>,
        vertex_flags: VertexFlag,
        allocator: Arc<Mutex<Allocator>>,
    ) -> Arc<Self> {
        let vertices_data = mesh
            .primitives
            .iter()
            .flat_map(|p| {
                log::debug!(
                    "Loading primitive: {} Indices, {} Vertices",
                    p.indices.len(),
                    p.vertices.len()
                );
                log::debug!("Vertex size: {}", VertexData::size(vertex_flags, true));
                p.vertices_data(vertex_flags)
            })
            .collect::<Vec<u8>>();

        let mut indices = Vec::new();

        for p in &mesh.primitives {
            let start = indices.len() as u32;
            for &idx in &p.indices {
                indices.push(idx + start);
            }
        }

        let vertices_size = vertices_data.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();

        let mut staging = Buffer::new(
            indices_size + vertices_size,
            BufferUsage::Staging,
            ctx.device.clone(),
            allocator.clone(),
        );

        let index_buffer = Buffer::new(
            indices_size,
            BufferUsage::Index,
            ctx.device.clone(),
            allocator.clone(),
        );
        let vertex_buffer = Buffer::new(
            vertices_size,
            BufferUsage::Vertex,
            ctx.device.clone(),
            allocator.clone(),
        );

        staging.copy(&vertices_data, 0);
        staging.copy(&indices, vertices_size);

        ctx.immediate_cmd.immediate(|cmd| {
            cmd.copy_buffer(&staging, &vertex_buffer, vertex_buffer.size, 0);
            cmd.copy_buffer(
                &staging,
                &index_buffer,
                index_buffer.size,
                vertex_buffer.size,
            );
        });

        Arc::new(MeshBuffer {
            index_buffer,
            vertex_buffer,
        })
    }
}
