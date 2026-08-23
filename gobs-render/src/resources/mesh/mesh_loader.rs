use gobs_core::logger;
use gobs_render_graph::GfxContext;
use gobs_render_hal::{
    AlignMode, BufferType, CommandBuffer, CommandQueueType, RenderHAL, VertexAttribute, VertexData,
};
use gobs_resource::{
    ResourceRegistry, {ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::{
    MeshProperties,
    resources::{BufferPool, Mesh, MeshData, MeshGeometry, MeshPath, MeshPrimitiveType},
};

pub struct MeshLoader {
    cmd: Box<dyn CommandBuffer>,
    buffer_pool: BufferPool,
    recording: bool,
}

impl MeshLoader {
    pub fn new(ctx: &mut GfxContext) -> Self {
        Self {
            cmd: ctx
                .hal_mut()
                .create_command_buffer("Mesh loader", CommandQueueType::Transfer),
            buffer_pool: BufferPool::new(),
            recording: false,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn start_recording(&mut self) {
        tracing::debug!(target: logger::RENDER, "Record mesh loading command");
        self.recording = true;

        self.cmd.reset();

        self.cmd.begin(0);
        self.cmd.begin_label("Upload buffer");
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn stop_recording(&mut self) {
        tracing::debug!(target: logger::RENDER, "Submit mesh loading command");
        self.cmd.end_label();
        self.cmd.end();
        self.cmd.submit_transfer();

        self.cmd.wait();

        self.buffer_pool.recycle_all();

        self.recording = false;
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load_data(&mut self, hal: &mut dyn RenderHAL, vertices: &[u8], indices: &[u32]) -> MeshData {
        let vertices_size = vertices.len();
        let indices_size = std::mem::size_of_val(indices);
        let staging_size = indices_size + vertices_size;

        if !self.recording {
            self.start_recording();
        }

        let staging = self
            .buffer_pool
            .allocate(hal, "staging", staging_size, BufferType::Staging);

        let vertex_view = hal.create_buffer("vertex", vertices_size, BufferType::Vertex);
        let index_view = hal.create_buffer("index", indices_size, BufferType::Index);

        hal.upload_buffer(staging.buffer, vertices, 0);
        hal.upload_buffer(
            staging.buffer,
            bytemuck::cast_slice(indices),
            vertices_size as u64,
        );

        self.cmd
            .copy_buffer_to_buffer(hal, staging.buffer, vertex_view, vertices_size, 0, 0);
        self.cmd.copy_buffer_to_buffer(
            hal,
            staging.buffer,
            index_view,
            indices_size,
            vertices_size as u64,
            0,
        );

        MeshData {
            ty: MeshPrimitiveType::Triangle,
            vertex_view,
            index_view,
            index_len: indices.len(),
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load_geometry(
        &mut self,
        hal: &mut dyn RenderHAL,
        geometry: &MeshGeometry,
        vertex_attributes: VertexAttribute,
    ) -> MeshData {
        tracing::debug!(target: logger::INIT, "Loading geometry for {} with layout {:?}", &geometry.name, vertex_attributes);
        let mut vertices = Vec::new();

        VertexData::copy_data(
            &geometry.vertices,
            vertex_attributes,
            &mut vertices,
            AlignMode::Scalar,
        );

        let indices = &geometry.indices;

        debug_assert!(!vertices.is_empty());
        debug_assert!(!indices.is_empty());

        self.load_data(hal, &vertices, indices)
    }
}

impl ResourceLoader<Mesh> for MeshLoader {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load<'a>(
        &mut self,
        hal: &mut (dyn RenderHAL + 'a),
        handle: &ResourceHandle<Mesh>,
        registry: &mut ResourceRegistry,
    ) -> Result<MeshData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        tracing::debug!(target: logger::RESOURCES, "Load mesh resource {}", properties.name());

        let data = match &properties.path {
            MeshPath::Default => todo!(),
            MeshPath::File(_) => todo!(),
            MeshPath::Bytes((vertices, indices)) => self.load_data(hal, vertices, indices),
            MeshPath::Mesh(geometry) => {
                self.load_geometry(hal, geometry, properties.vertex_attributes)
            }
        };

        Ok(data)
    }

    fn unload<'a>(
        &mut self,
        hal: &mut (dyn RenderHAL + 'a),
        data: MeshData,
        _properties: MeshProperties,
    ) {
        hal.destroy_buffer(data.vertex_view);
        hal.destroy_buffer(data.index_view);
    }

    fn flush(&mut self) {
        if self.recording {
            self.stop_recording();
        }
    }
}
