use gobs_core::logger;
use gobs_render_hal::{BufferType, CommandBuffer, CommandQueueType, RenderHAL, VertexAttribute};
use gobs_resource::{
    ResourceRegistry, {Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::{
    GfxContext,
    resources::{Mesh, MeshData, MeshGeometry, MeshPath, MeshPrimitiveType},
};

pub struct MeshLoader {
    cmd: Box<dyn CommandBuffer>,
}

const STAGING_BUFFER_SIZE: usize = 1_048_576;

impl MeshLoader {
    pub fn new(ctx: &mut GfxContext) -> Self {
        Self {
            cmd: ctx
                .hal
                .create_command_buffer("Mesh loader", CommandQueueType::Transfer),
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load_geometry(
        &mut self,
        hal: &mut dyn RenderHAL,
        geometry: &MeshGeometry,
        vertex_attributes: &VertexAttribute,
    ) -> MeshData {
        tracing::debug!(target: logger::INIT, "Loading geometry for {} with layout {:?}", &geometry.name, vertex_attributes);
        let mut vertices = Vec::new();

        // TODO: hot path
        let alignment = vertex_attributes.alignment();

        for vertice in &geometry.vertices {
            vertice.copy_data(vertex_attributes, alignment, &mut vertices);
        }

        let indices = &geometry.indices;

        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();
        let staging_size = indices_size + vertices_size;

        // TODO: drop or reuse staging buffer
        let staging = hal.create_buffer(
            "staging",
            staging_size.max(STAGING_BUFFER_SIZE),
            BufferType::Staging,
        );

        let vertex_view = hal.create_buffer("vertex", vertices_size, BufferType::Vertex);
        let index_view = hal.create_buffer("index", indices_size, BufferType::Index);

        hal.upload_buffer(staging, &vertices, 0);
        hal.upload_buffer(staging, bytemuck::cast_slice(indices), vertices_size as u64);

        self.cmd.run_immediate_mut("Upload buffer", &mut |cmd| {
            cmd.copy_buffer_to_buffer(hal, staging, vertex_view, vertices_size, 0, 0);
            cmd.copy_buffer_to_buffer(
                hal,
                staging,
                index_view,
                indices_size,
                vertices_size as u64,
                0,
            );
        });

        MeshData {
            ty: MeshPrimitiveType::Triangle,
            vertex_view,
            index_view,
            index_len: indices.len(),
        }
    }
}

impl ResourceLoader<Mesh> for MeshLoader {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load(
        &mut self,
        hal: &mut Box<dyn RenderHAL>,
        handle: &ResourceHandle<Mesh>,
        registry: &mut ResourceRegistry,
    ) -> Result<MeshData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        tracing::debug!(target: logger::RESOURCES, "Load mesh resource {}", properties.name());

        let data = match &properties.path {
            MeshPath::Default => todo!(),
            MeshPath::File(_) => todo!(),
            MeshPath::Bytes(_) => todo!(),
            MeshPath::Mesh(geometry) => {
                self.load_geometry(hal.as_mut(), geometry, &properties.vertex_attributes)
            }
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<Mesh>) {
        // drop resource
    }
}
