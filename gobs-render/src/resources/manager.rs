use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use gobs_gfx::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, Buffer, BufferUsage, Command, Device,
    ImageLayout, Pipeline, Renderer,
};
use gobs_resource::{
    geometry::{BoundingBox, Mesh, VertexData},
    material::TextureId,
};

use crate::{
    context::Context,
    material::{MaterialInstance, MaterialInstanceId},
    model::Model,
    pass::{PassId, RenderPass},
    ModelId,
};

use super::GpuTexture;

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Debug)]
pub struct GPUMesh<R: Renderer> {
    pub model: Arc<Model<R>>,
    pub ty: PrimitiveType,
    pub material: Option<Arc<MaterialInstance<R>>>,
    pub material_binding: Option<R::BindingGroup>,
    pub vertex_buffer: Arc<R::Buffer>,
    pub index_buffer: Arc<R::Buffer>,
    pub vertices_offset: u64,
    pub indices_offset: usize,
    pub indices_len: usize,
}

impl<R: Renderer> Clone for GPUMesh<R> {
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

pub struct MeshResourceManager<R: Renderer> {
    pub mesh_data: HashMap<ResourceKey, Vec<GPUMesh<R>>>,
    pub transient_mesh_data: Vec<HashMap<ResourceKey, Vec<GPUMesh<R>>>>,
    pub material_bindings: HashMap<MaterialInstanceId, R::BindingGroup>,
    pub textures: HashMap<TextureId, GpuTexture<R>>,
}

impl<R: Renderer> MeshResourceManager<R> {
    pub fn new(ctx: &Context<R>) -> Self {
        let transient_mesh_data = (0..(ctx.frames_in_flight + 1))
            .map(|_| HashMap::new())
            .collect();
        Self {
            mesh_data: HashMap::new(),
            transient_mesh_data,
            material_bindings: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    fn debug_stats(&self) {
        tracing::debug!(target: "render", "Meshes: {}", self.mesh_data.keys().len());
        tracing::debug!(target: "render", "Transient: {}", self.transient_mesh_data[0].keys().len());
        tracing::debug!(target: "render", "Bindings: {}", self.material_bindings.keys().len());
    }

    pub fn new_frame(&mut self, ctx: &Context<R>) -> usize {
        self.debug_stats();
        let frame_id = ctx.frame_id();

        self.transient_mesh_data[frame_id].clear();
        frame_id
    }

    pub fn add_object(
        &mut self,
        ctx: &Context<R>,
        object: Arc<Model<R>>,
        pass: Arc<dyn RenderPass<R>>,
        transient: bool,
    ) -> &[GPUMesh<R>] {
        let key = (object.id, pass.id());

        if !transient {
            let cached = self.mesh_data.contains_key(&key);

            if !cached {
                let data = self.load_object(ctx, object.clone(), pass.clone());
                self.mesh_data.insert(key, data);
            }

            self.mesh_data.get(&key).expect("Get mesh data")
        } else {
            let frame_id = ctx.frame_id();
            let data = self.load_object(ctx, object.clone(), pass.clone());
            self.transient_mesh_data[frame_id].insert(key, data);

            self.transient_mesh_data[frame_id]
                .get(&key)
                .expect("Get transient mesh data")
        }
    }

    pub fn add_bounding_box(
        &mut self,
        ctx: &Context<R>,
        bounding_box: BoundingBox,
        pass: Arc<dyn RenderPass<R>>,
    ) -> &[GPUMesh<R>] {
        let frame_id = ctx.frame_id();
        let (model, data) = self.load_box(ctx, bounding_box, pass.clone());
        let key = (model.id, pass.id());

        self.transient_mesh_data[frame_id].insert(key, data);

        self.transient_mesh_data[frame_id]
            .get(&key)
            .expect("Get transient box data")
    }

    fn load_object(
        &mut self,
        ctx: &Context<R>,
        model: Arc<Model<R>>,
        pass: Arc<dyn RenderPass<R>>,
    ) -> Vec<GPUMesh<R>> {
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

        let (vertex_buffer, index_buffer) = Self::upload_vertices(ctx, &vertices, &indices);

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
        ctx: &Context<R>,
        bounding_box: BoundingBox,
        pass: Arc<dyn RenderPass<R>>,
    ) -> (Arc<Model<R>>, Vec<GPUMesh<R>>) {
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
                .padding(true)
                .build();

            mesh = mesh.vertex(vertex_data);
        }

        let mesh = mesh.build();

        let model = Model::builder("box").mesh(mesh, None).build();

        (model.clone(), self.load_object(ctx, model, pass))
    }

    fn load_material(
        &mut self,
        ctx: &Context<R>,
        material: Option<Arc<MaterialInstance<R>>>,
    ) -> Option<R::BindingGroup> {
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

    fn load_texture(&mut self, ctx: &Context<R>, material: &MaterialInstance<R>) {
        for texture in &material.textures {
            let key = texture.id;

            if !self.textures.contains_key(&key) {
                self.textures
                    .insert(key, GpuTexture::new(ctx, texture.clone()));
            }
        }
    }

    fn upload_vertices(
        ctx: &Context<R>,
        vertices: &[u8],
        indices: &[u32],
    ) -> (Arc<R::Buffer>, Arc<R::Buffer>) {
        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();

        let mut staging = R::Buffer::new(
            "staging",
            indices_size + vertices_size,
            BufferUsage::Staging,
            &ctx.device,
        );

        let vertex_buffer =
            R::Buffer::new("vertex", vertices_size, BufferUsage::Vertex, &ctx.device);
        let index_buffer = R::Buffer::new("index", indices_size, BufferUsage::Index, &ctx.device);

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

        (Arc::new(vertex_buffer), Arc::new(index_buffer))
    }
}
