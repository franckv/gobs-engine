use std::sync::Arc;

use gobs_material::MaterialInstance;
use gobs_render::context::Context;
use gobs_vulkan::buffer::{Buffer, BufferUsage};

use crate::mesh::Mesh;

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

pub struct Model {
    pub name: String,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub primitives: Vec<Primitive>,
    pub meshes: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<MaterialInstance>>,
}

impl Model {
    pub fn new(
        ctx: &Context,
        name: &str,
        meshes: &[Arc<Mesh>],
        materials: &[Arc<MaterialInstance>],
    ) -> Arc<Self> {
        log::debug!("New model");

        let mut indices = Vec::new();
        let mut vertices = Vec::new();
        let mut primitives = Vec::new();

        for mesh in meshes {
            let start_idx = vertices.len() as u32;
            let offset = indices.len();

            for vertex in &mesh.vertices {
                vertices.push(vertex);
            }
            for index in &mesh.indices {
                indices.push(start_idx + index);
            }
            primitives.push(Primitive::new(
                PrimitiveType::Triangle,
                offset,
                indices.len() - offset,
                0,
            ));
        }

        let vertices_data = vertices
            .iter()
            .flat_map(|v| v.raw(materials[0].material.vertex_flags))
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
            name: name.to_string(),
            index_buffer,
            vertex_buffer,
            primitives,
            meshes: meshes.to_vec(),
            materials: materials.to_vec(),
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        log::debug!("Drop model");
    }
}
