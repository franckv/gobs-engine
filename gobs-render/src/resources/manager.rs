use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use gobs_gfx::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, Buffer, BufferUsage, Command, Device,
    GfxBindingGroup, GfxBuffer, ImageLayout, Pipeline,
};
use gobs_resource::{
    geometry::{BoundingBox, Mesh, VertexData},
    material::TextureId,
};

use crate::{
    context::Context,
    material::{MaterialInstance, MaterialInstanceId},
    model::Model,
    pass::PassId,
    renderable::RenderableLifetime,
    resources::{Allocator, GpuTexture},
    ModelId, RenderPass,
};

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Debug)]
pub struct GPUMesh {
    pub model: Arc<Model>,
    pub ty: PrimitiveType,
    pub material: Option<Arc<MaterialInstance>>,
    pub material_binding: Option<GfxBindingGroup>,
    pub vertex_buffer: Arc<GfxBuffer>,
    pub index_buffer: Arc<GfxBuffer>,
    pub vertices_offset: u64,
    pub indices_offset: usize,
    pub indices_len: usize,
}

impl Clone for GPUMesh {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            ty: self.ty.clone(),
            material: self.material.clone(),
            material_binding: self.material_binding.clone(),
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            vertices_offset: self.vertices_offset.clone(),
            indices_offset: self.indices_offset.clone(),
            indices_len: self.indices_len.clone(),
        }
    }
}

type ResourceKey = (ModelId, PassId);

const STAGING_BUFFER_SIZE: usize = 1_048_576;

pub struct MeshResourceManager {
    pub mesh_data: HashMap<ResourceKey, Vec<GPUMesh>>,
    pub transient_mesh_data: Vec<HashMap<ResourceKey, Vec<GPUMesh>>>,
    pub material_bindings: HashMap<MaterialInstanceId, GfxBindingGroup>,
    pub textures: HashMap<TextureId, GpuTexture>,
    pub buffer_pool: Allocator<BufferUsage, GfxBuffer>,
}

impl MeshResourceManager {
    pub fn new(ctx: &Context) -> Self {
        let transient_mesh_data = (0..(ctx.frames_in_flight + 1))
            .map(|_| HashMap::new())
            .collect();
        Self {
            mesh_data: HashMap::new(),
            transient_mesh_data,
            material_bindings: HashMap::new(),
            textures: HashMap::new(),
            buffer_pool: Allocator::new(),
        }
    }

    fn debug_stats(&self) {
        tracing::debug!(target: "render", "Meshes: {}", self.mesh_data.keys().len());
        tracing::debug!(target: "render", "Transient: {}", self.transient_mesh_data[0].keys().len());
        tracing::debug!(target: "render", "Bindings: {}", self.material_bindings.keys().len());
    }

    pub fn new_frame(&mut self, ctx: &Context) -> usize {
        self.debug_stats();
        let frame_id = ctx.frame_id();

        for (_, mut data) in self.transient_mesh_data[frame_id].drain() {
            for mesh in data.drain(..) {
                let index = Arc::into_inner(mesh.index_buffer);
                if let Some(buffer) = index {
                    self.buffer_pool.recycle(buffer);
                }
                let vertex = Arc::into_inner(mesh.vertex_buffer);
                if let Some(buffer) = vertex {
                    self.buffer_pool.recycle(buffer);
                }
            }
        }
        self.transient_mesh_data[frame_id].clear();
        frame_id
    }

    #[tracing::instrument(target = "resources", skip_all, level = "debug")]
    pub fn add_object(
        &mut self,
        ctx: &Context,
        object: Arc<Model>,
        pass: RenderPass,
        lifetime: RenderableLifetime,
    ) -> &[GPUMesh] {
        let key = (object.id, pass.id());

        if lifetime == RenderableLifetime::Static {
            let cached = self.mesh_data.contains_key(&key);

            if !cached {
                let data = self.load_object(ctx, object.clone(), pass.clone(), lifetime);
                self.mesh_data.insert(key, data);
            }

            self.mesh_data.get(&key).expect("Get mesh data")
        } else {
            let frame_id = ctx.frame_id();
            let data = self.load_object(ctx, object.clone(), pass.clone(), lifetime);
            self.transient_mesh_data[frame_id].insert(key, data);

            self.transient_mesh_data[frame_id]
                .get(&key)
                .expect("Get transient mesh data")
        }
    }

    pub fn add_bounding_box(
        &mut self,
        ctx: &Context,
        bounding_box: BoundingBox,
        pass: RenderPass,
        lifetime: RenderableLifetime,
    ) -> &[GPUMesh] {
        let frame_id = ctx.frame_id();
        let (model, data) = self.load_box(ctx, bounding_box, pass.clone(), lifetime);
        let key = (model.id, pass.id());

        self.transient_mesh_data[frame_id].insert(key, data);

        self.transient_mesh_data[frame_id]
            .get(&key)
            .expect("Get transient box data")
    }

    #[tracing::instrument(target = "resources", skip_all, level = "debug")]
    fn load_object(
        &mut self,
        ctx: &Context,
        model: Arc<Model>,
        pass: RenderPass,
        lifetime: RenderableLifetime,
    ) -> Vec<GPUMesh> {
        let mut gpu_meshes = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        let mut vertices_offset;
        let mut indices_offset;

        let mut vertex_data = Vec::new();

        for (mesh, material_id) in &model.meshes {
            vertices_offset = vertices.len() as u64;
            indices_offset = indices.len();

            tracing::trace!("Vertex offset: {}, {}", vertices_offset, indices_offset);

            let vertex_flags = match pass.vertex_flags() {
                Some(vertex_flags) => vertex_flags,
                None => model.materials[material_id].vertex_flags(),
            };
            // TODO: hot path
            let alignment = vertex_flags.alignment();
            for vertice in &mesh.vertices {
                vertice.copy_data(vertex_flags, alignment, &mut vertices);
            }
            for index in &mesh.indices {
                indices.push(*index);
            }

            vertex_data.push((
                material_id,
                vertices_offset,
                indices_offset,
                indices.len() - indices_offset,
            ))
        }

        let (vertex_buffer, index_buffer) =
            self.upload_vertices(ctx, &vertices, &indices, lifetime);

        for (&material_id, vertices_offset, indices_offset, indices_len) in vertex_data {
            let material = model.materials.get(&material_id).cloned();

            // TODO: manage transient material
            let material_binding = self.load_material(ctx, material.clone());

            gpu_meshes.push(GPUMesh {
                model: model.clone(),
                ty: PrimitiveType::Triangle,
                material,
                material_binding,
                vertex_buffer: vertex_buffer.clone(),
                index_buffer: index_buffer.clone(),
                vertices_offset,
                indices_offset,
                indices_len,
            });
        }

        gpu_meshes
    }

    fn load_box(
        &mut self,
        ctx: &Context,
        bounding_box: BoundingBox,
        pass: RenderPass,
        lifetime: RenderableLifetime,
    ) -> (Arc<Model>, Vec<GPUMesh>) {
        tracing::debug!("New box");

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
                .padding(ctx.vertex_padding)
                .build();

            mesh = mesh.vertex(vertex_data);
        }

        let mesh = mesh.build();

        let model = Model::builder("box").mesh(mesh, None).build();

        (model.clone(), self.load_object(ctx, model, pass, lifetime))
    }

    fn load_material(
        &mut self,
        ctx: &Context,
        material: Option<Arc<MaterialInstance>>,
    ) -> Option<GfxBindingGroup> {
        if let Some(ref material) = material {
            tracing::debug!("Save binding for {}", material.id);
            self.load_texture(ctx, &material);

            match self.material_bindings.entry(material.id) {
                Entry::Vacant(e) => {
                    if !material.textures.is_empty() {
                        tracing::debug!(
                            "Create material binding for pipeline: {}",
                            material.pipeline().id()
                        );
                        let binding = material
                            .material
                            .pipeline
                            .create_binding_group(BindingGroupType::MaterialData)
                            .unwrap();
                        let mut updater = binding.update();
                        for texture in &material.textures {
                            let gpu_texture = self.textures.get(&texture.id).expect("Load texture");
                            updater = updater
                                .bind_sampled_image(gpu_texture.image(), ImageLayout::Shader)
                                .bind_sampler(gpu_texture.sampler());
                        }
                        updater.end();

                        Some(e.insert(binding).clone())
                    } else {
                        None
                    }
                }
                Entry::Occupied(e) => Some(e.get().clone()),
            }
        } else {
            None
        }
    }

    fn load_texture(&mut self, ctx: &Context, material: &MaterialInstance) {
        for texture in &material.textures {
            let key = texture.id;

            if !self.textures.contains_key(&key) {
                self.textures
                    .insert(key, GpuTexture::new(ctx, texture.clone()));
            }
        }
    }

    #[tracing::instrument(target = "resources", skip_all, level = "debug")]
    fn upload_vertices(
        &mut self,
        ctx: &Context,
        vertices: &[u8],
        indices: &[u32],
        _lifetime: RenderableLifetime,
    ) -> (Arc<GfxBuffer>, Arc<GfxBuffer>) {
        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();

        let staging_size = indices_size + vertices_size;

        let mut staging = self.buffer_pool.allocate(
            ctx,
            "staging",
            staging_size.max(STAGING_BUFFER_SIZE),
            BufferUsage::Staging,
        );
        let mut vertex_buffer =
            self.buffer_pool
                .allocate(ctx, "vertex", vertices_size, BufferUsage::Vertex);
        let mut index_buffer =
            self.buffer_pool
                .allocate(ctx, "index", indices_size, BufferUsage::Index);

        staging.copy(&vertices, 0);
        staging.copy(&indices, vertices_size);

        ctx.device.run_transfer_mut(|cmd| {
            cmd.begin_label("Upload buffer");
            cmd.copy_buffer(&staging, &mut vertex_buffer, vertices_size, 0);
            cmd.copy_buffer(&staging, &mut index_buffer, indices_size, vertices_size);
            cmd.end_label();
        });

        self.buffer_pool.recycle(staging);

        (Arc::new(vertex_buffer), Arc::new(index_buffer))
    }
}
