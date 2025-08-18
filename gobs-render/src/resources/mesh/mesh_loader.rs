use std::sync::Arc;

use gobs_core::{
    logger,
    memory::allocator::{AllocationError, Allocator},
};
use gobs_gfx::{Buffer, BufferUsage, Command, CommandQueueType, GfxBuffer, GfxCommand, GfxDevice};
use gobs_resource::{
    geometry::{MeshGeometry, VertexAttribute},
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::resources::{Mesh, MeshData, MeshPath, MeshPrimitiveType};

pub struct MeshLoader {
    device: Arc<GfxDevice>,
    pub buffer_pool: Allocator<GfxDevice, BufferUsage, GfxBuffer>,
    cmd: GfxCommand,
}

const STAGING_BUFFER_SIZE: usize = 1_048_576;

impl MeshLoader {
    pub fn new(device: Arc<GfxDevice>) -> Self {
        Self {
            device: device.clone(),
            buffer_pool: Allocator::new(),
            cmd: GfxCommand::new(&device, "Mesh loader", CommandQueueType::Transfer),
        }
    }

    pub fn load_geometry(
        &mut self,
        geometry: &MeshGeometry,
        vertex_attributes: &VertexAttribute,
    ) -> Result<MeshData, AllocationError> {
        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        let vertices_offset = vertices.len();
        let indices_offset = indices.len();

        tracing::trace!(target: logger::RESOURCES, "Vertex offset: {}, {}", vertices_offset, indices_offset);

        // TODO: hot path
        let alignment = vertex_attributes.alignment();
        for vertice in &geometry.vertices {
            vertice.copy_data(vertex_attributes, alignment, &mut vertices);
        }
        for index in &geometry.indices {
            indices.push(*index);
        }

        let (vertex_buffer, index_buffer) = self.upload_vertices(&vertices, &indices)?;

        Ok(MeshData {
            ty: MeshPrimitiveType::Triangle,
            vertex_buffer: vertex_buffer.clone(),
            index_buffer: index_buffer.clone(),
            vertices_offset: vertices_offset as u64,
            vertices_len: vertices.len() - vertices_offset,
            vertices_count: geometry.vertices.len(),
            indices_offset,
            indices_len: indices.len() - indices_offset,
        })
    }

    #[tracing::instrument(target = "resources", skip_all, level = "trace")]
    fn upload_vertices(
        &mut self,
        vertices: &[u8],
        indices: &[u32],
    ) -> Result<(Arc<GfxBuffer>, Arc<GfxBuffer>), AllocationError> {
        let vertices_size = vertices.len();
        let indices_size = std::mem::size_of_val(indices);

        let staging_size = indices_size + vertices_size;

        let staging = self.buffer_pool.allocate(
            &self.device,
            "staging",
            staging_size.max(STAGING_BUFFER_SIZE),
            BufferUsage::Staging,
        )?;
        let staging_id = staging.id();

        let mut vertex_buffer =
            GfxBuffer::new("vertex", vertices_size, BufferUsage::Vertex, &self.device);
        let mut index_buffer =
            GfxBuffer::new("index", indices_size, BufferUsage::Index, &self.device);

        staging.copy(vertices, 0);
        staging.copy(indices, vertices_size);

        self.cmd.run_immediate_mut("Upload buffer", |cmd| {
            cmd.copy_buffer(staging, &mut vertex_buffer, vertices_size, 0);
            cmd.copy_buffer(staging, &mut index_buffer, indices_size, vertices_size);
        });

        self.buffer_pool.recycle(&staging_id);

        Ok((Arc::new(vertex_buffer), Arc::new(index_buffer)))
    }
}

impl ResourceLoader<Mesh> for MeshLoader {
    fn load(
        &mut self,
        handle: &ResourceHandle<Mesh>,
        parameter: &VertexAttribute,
        registry: &mut ResourceRegistry,
    ) -> Result<MeshData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        tracing::debug!(target: logger::RESOURCES, "Load mesh resource {}", properties.name());

        let data = match &properties.path {
            MeshPath::Default => todo!(),
            MeshPath::File(_) => todo!(),
            MeshPath::Bytes(_) => todo!(),
            MeshPath::Mesh(geometry) => self.load_geometry(geometry, parameter)?,
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<Mesh>) {
        // drop resource
    }
}
