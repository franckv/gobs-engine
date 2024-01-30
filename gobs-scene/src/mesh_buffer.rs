use std::sync::Arc;

use gobs_render::context::Context;
use gobs_vulkan::buffer::{Buffer, BufferUsage};

use crate::geometry::{mesh::Mesh, vertex::VertexFlag};

pub struct MeshBuffer {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
}

impl MeshBuffer {
    pub fn new(ctx: &Context, mesh: Arc<Mesh>, vertex_flags: VertexFlag) -> Arc<Self> {
        let vertices_data = mesh.vertices_data(vertex_flags);
        let vertices_size = vertices_data.len();
        let indices_size = mesh.indices.len() * std::mem::size_of::<u32>();

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
        staging.copy(&mesh.indices, vertices_size);

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

        Arc::new(MeshBuffer {
            index_buffer,
            vertex_buffer,
        })
    }
}
