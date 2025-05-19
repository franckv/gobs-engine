use std::sync::Arc;

use gobs_gfx::{Buffer, BufferUsage, Command, Device, GfxBuffer, GfxDevice};
use gobs_resource::{
    geometry::{MeshGeometry, VertexAttribute},
    resource::{Resource, ResourceLoader},
};

use crate::{
    manager::Allocator,
    resources::{Mesh, MeshData, MeshPath, MeshPrimitiveType, MeshProperties},
};

pub struct MeshLoader {
    device: Arc<GfxDevice>,
    pub buffer_pool: Allocator<BufferUsage, GfxBuffer>,
}

const STAGING_BUFFER_SIZE: usize = 1_048_576;

impl MeshLoader {
    pub fn new(device: Arc<GfxDevice>) -> Self {
        Self {
            device,
            buffer_pool: Allocator::new(),
        }
    }

    pub fn load_geometry(
        &mut self,
        geometry: &MeshGeometry,
        vertex_attributes: &VertexAttribute,
    ) -> MeshData {
        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        let vertices_offset = vertices.len();
        let indices_offset = indices.len();

        tracing::trace!(target: "resources", "Vertex offset: {}, {}", vertices_offset, indices_offset);

        // TODO: hot path
        let alignment = vertex_attributes.alignment();
        for vertice in &geometry.vertices {
            vertice.copy_data(vertex_attributes, alignment, &mut vertices);
        }
        for index in &geometry.indices {
            indices.push(*index);
        }

        let (vertex_buffer, index_buffer) = self.upload_vertices(&vertices, &indices);

        MeshData {
            ty: MeshPrimitiveType::Triangle,
            vertex_buffer: vertex_buffer.clone(),
            index_buffer: index_buffer.clone(),
            vertices_offset: vertices_offset as u64,
            vertices_len: vertices.len() - vertices_offset,
            vertices_count: geometry.vertices.len(),
            indices_offset,
            indices_len: indices.len() - indices_offset,
        }
    }

    #[tracing::instrument(target = "resources", skip_all, level = "trace")]
    fn upload_vertices(
        &mut self,
        vertices: &[u8],
        indices: &[u32],
    ) -> (Arc<GfxBuffer>, Arc<GfxBuffer>) {
        let vertices_size = vertices.len();
        let indices_size = std::mem::size_of_val(indices);

        let staging_size = indices_size + vertices_size;

        let mut staging = self.buffer_pool.allocate(
            &self.device,
            "staging",
            staging_size.max(STAGING_BUFFER_SIZE),
            BufferUsage::Staging,
        );
        let mut vertex_buffer =
            self.buffer_pool
                .allocate(&self.device, "vertex", vertices_size, BufferUsage::Vertex);
        let mut index_buffer =
            self.buffer_pool
                .allocate(&self.device, "index", indices_size, BufferUsage::Index);

        staging.copy(vertices, 0);
        staging.copy(indices, vertices_size);

        self.device.run_transfer_mut(|cmd| {
            cmd.begin_label("Upload buffer");
            cmd.copy_buffer(&staging, &mut vertex_buffer, vertices_size, 0);
            cmd.copy_buffer(&staging, &mut index_buffer, indices_size, vertices_size);
            cmd.end_label();
        });

        self.buffer_pool.recycle(staging);

        (Arc::new(vertex_buffer), Arc::new(index_buffer))
    }
}

impl ResourceLoader<Mesh> for MeshLoader {
    fn load(&mut self, properties: &mut MeshProperties, parameter: &VertexAttribute) -> MeshData {
        match &properties.path {
            MeshPath::Default => todo!(),
            MeshPath::File(_) => todo!(),
            MeshPath::Bytes(_) => todo!(),
            MeshPath::Mesh(geometry) => self.load_geometry(geometry, parameter),
        }
    }

    fn unload(&mut self, _resource: Resource<Mesh>) {
        // drop resource
    }
}
