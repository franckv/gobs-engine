use std::sync::{Arc, Mutex};

use gpu_allocator::vulkan::Allocator;

use gobs_core::geometry::{mesh::Mesh, vertex::VertexFlag};
use gobs_render::context::Context;
use gobs_vulkan::buffer::{Buffer, BufferUsage};

pub struct MeshResource {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
}

impl MeshResource {
    pub fn new(
        ctx: &Context,
        mesh: Arc<Mesh>,
        vertex_flags: VertexFlag,
        allocator: Arc<Mutex<Allocator>>,
    ) -> Arc<Self> {
        let vertices_data = mesh.vertices_data(vertex_flags);
        let vertices_size = vertices_data.len();

        let indices_size = mesh.indices.len() * std::mem::size_of::<u32>();

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
        staging.copy(&mesh.indices, vertices_size);

        ctx.immediate_cmd.immediate(|cmd| {
            cmd.copy_buffer(&staging, &vertex_buffer, vertex_buffer.size, 0);
            cmd.copy_buffer(
                &staging,
                &index_buffer,
                index_buffer.size,
                vertex_buffer.size,
            );
        });

        Arc::new(MeshResource {
            index_buffer,
            vertex_buffer,
        })
    }
}
