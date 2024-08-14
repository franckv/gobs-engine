use std::sync::Arc;

use gobs_gfx::{Buffer, BufferUsage, Command, Device};

use crate::{
    context::Context,
    geometry::{BoundingBox, Mesh, Model, VertexData},
    material::MaterialInstanceId,
    pass::RenderPass,
    GfxBuffer,
};

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Clone, Copy, Debug)]
pub struct Primitive {
    pub ty: PrimitiveType,
    pub vertex_offset: u64,
    pub index_offset: usize,
    pub len: usize,
    pub material: Option<MaterialInstanceId>,
}

impl Primitive {
    pub fn new(
        ty: PrimitiveType,
        vertex_offset: u64,
        index_offset: usize,
        len: usize,
        material: Option<MaterialInstanceId>,
    ) -> Self {
        Primitive {
            ty,
            vertex_offset,
            index_offset,
            len,
            material,
        }
    }
}

pub struct ModelResource {
    pub model: Arc<Model>,
    pub index_buffer: GfxBuffer,
    pub vertex_buffer: GfxBuffer,
    pub primitives: Vec<Primitive>,
}

impl ModelResource {
    pub fn new(ctx: &Context, model: Arc<Model>, pass: Arc<dyn RenderPass>) -> Arc<Self> {
        log::debug!("New model");

        let (vertices, indices, primitives) = Self::compute_vertices(model.clone(), pass);

        log::debug!(
            "Upload {} vertices {} indices",
            vertices.len(),
            indices.len()
        );

        let (vertex_buffer, index_buffer) = Self::upload_vertices(ctx, &vertices, &indices);

        Arc::new(Self {
            model,
            index_buffer,
            vertex_buffer,
            primitives,
        })
    }

    pub fn new_box(
        ctx: &Context,
        bounding_box: BoundingBox,
        pass: Arc<dyn RenderPass>,
    ) -> Arc<Self> {
        log::debug!("New box");

        let (left, bottom, back) = bounding_box.bottom_left().into();
        let (right, top, front) = bounding_box.top_right().into();

        let v = [
            [left, top, front],
            [right, top, front],
            [left, bottom, front],
            [right, bottom, front],
            [left, top, back],
            [right, top, back],
            [left, bottom, back],
            [right, bottom, back],
        ];

        let vi = [
            3, 4, 2, 3, 2, 1, // F
            8, 7, 5, 8, 5, 6, // B
            7, 3, 1, 7, 1, 5, // L
            4, 8, 6, 4, 6, 2, // R
            1, 2, 6, 1, 6, 5, // U
            7, 8, 4, 7, 4, 3, // D
        ];

        let mut mesh = Mesh::builder("bounds");

        for i in 0..vi.len() {
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .padding(true)
                .build();

            mesh = mesh.vertex(vertex_data);
        }

        let mesh = mesh.build();

        let model = Model::builder("box").mesh(mesh, None).build();

        let (vertices, indices, primitives) = Self::compute_vertices(model.clone(), pass);

        let (vertex_buffer, index_buffer) = Self::upload_vertices(ctx, &vertices, &indices);

        Arc::new(Self {
            model,
            index_buffer,
            vertex_buffer,
            primitives,
        })
    }

    fn compute_vertices(
        model: Arc<Model>,
        pass: Arc<dyn RenderPass>,
    ) -> (Vec<u8>, Vec<u32>, Vec<Primitive>) {
        let mut indices = Vec::new();
        let mut vertices = Vec::new();
        let mut primitives = Vec::new();

        let mut vertex_offset = 0;

        for (mesh, material_id) in &model.meshes {
            let offset = indices.len();

            let vertex_flags = match pass.vertex_flags() {
                Some(vertex_flags) => vertex_flags,
                None => model.materials[material_id].vertex_flags(),
            };
            // TODO: hot path
            let alignment = vertex_flags.alignment();
            for vertice in &mesh.vertices {
                vertices.append(&mut vertice.raw(vertex_flags, alignment));
            }
            for index in &mesh.indices {
                indices.push(*index);
            }
            primitives.push(Primitive::new(
                PrimitiveType::Triangle,
                vertex_offset,
                offset,
                indices.len() - offset,
                Some(*material_id),
            ));
            vertex_offset = vertices.len() as u64;

            log::trace!("Vertex offset: {}", vertex_offset);
        }

        (vertices, indices, primitives)
    }

    fn upload_vertices(ctx: &Context, vertices: &[u8], indices: &[u32]) -> (GfxBuffer, GfxBuffer) {
        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();

        let mut staging = GfxBuffer::new(
            "staging",
            indices_size + vertices_size,
            BufferUsage::Staging,
            &ctx.device,
        );

        let index_buffer = GfxBuffer::new("index", indices_size, BufferUsage::Index, &ctx.device);
        let vertex_buffer =
            GfxBuffer::new("vertex", vertices_size, BufferUsage::Vertex, &ctx.device);

        staging.copy(&vertices, 0);
        staging.copy(&indices, vertices_size);

        ctx.device.run_immediate(|cmd| {
            cmd.begin_label("Upload buffer");

            cmd.copy_buffer(&staging, &vertex_buffer, vertex_buffer.size(), 0);
            cmd.copy_buffer(
                &staging,
                &index_buffer,
                index_buffer.size(),
                vertex_buffer.size(),
            );
            cmd.end_label();
        });

        (vertex_buffer, index_buffer)
    }
}
