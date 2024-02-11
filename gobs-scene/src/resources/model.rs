use std::sync::Arc;

use gobs_render::{
    context::Context, geometry::Model, material::MaterialInstanceId, pass::RenderPass,
};
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
    pub material: MaterialInstanceId,
}

impl Primitive {
    pub fn new(ty: PrimitiveType, offset: usize, len: usize, material: MaterialInstanceId) -> Self {
        Primitive {
            ty,
            offset,
            len,
            material,
        }
    }
}

pub struct ModelResource {
    pub model: Arc<Model>,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub primitives: Vec<Primitive>,
}

impl ModelResource {
    pub fn new(ctx: &Context, model: Arc<Model>, pass: Arc<dyn RenderPass>) -> Arc<Self> {
        log::debug!("New model");

        let mut indices = Vec::new();
        let mut vertices = Vec::new();
        let mut primitives = Vec::new();

        let mut start_idx = 0;

        for (mesh, material_id) in &model.meshes {
            let offset = indices.len();

            let vertex_flags = match pass.vertex_flags() {
                Some(vertex_flags) => vertex_flags,
                None => model.materials[material_id].vertex_flags(),
            };
            for vertice in &mesh.vertices {
                vertices.append(&mut vertice.raw(vertex_flags));
            }
            for index in &mesh.indices {
                indices.push(start_idx + index);
            }
            start_idx += mesh.vertices.len() as u32;
            primitives.push(Primitive::new(
                PrimitiveType::Triangle,
                offset,
                indices.len() - offset,
                *material_id,
            ));
        }

        let vertices_size = vertices.len();
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

        staging.copy(&vertices, 0);
        staging.copy(&indices, vertices_size);

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

        Arc::new(Self {
            model,
            index_buffer,
            vertex_buffer,
            primitives,
        })
    }
}
