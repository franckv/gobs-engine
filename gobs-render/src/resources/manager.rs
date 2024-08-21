use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use gobs_gfx::{BindingGroupType, Buffer, BufferUsage, Command, Device, ImageLayout, Pipeline};
use gobs_resource::{
    geometry::{BoundingBox, Mesh, VertexData},
    material::TextureId,
};

use crate::{
    context::Context,
    material::{MaterialInstance, MaterialInstanceId},
    pass::{PassId, RenderPass},
    GfxBindingGroup, GfxBuffer, Model, ModelId,
};

use super::GpuTexture;

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Triangle,
}

#[derive(Clone, Debug)]
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

type ResourceKey = (ModelId, PassId);

pub struct MeshResourceManager {
    pub mesh_data: HashMap<ResourceKey, Vec<GPUMesh>>,
    pub transient_mesh_data: Vec<HashMap<ResourceKey, Vec<GPUMesh>>>,
    pub material_bindings: HashMap<MaterialInstanceId, GfxBindingGroup>,
    pub textures: HashMap<TextureId, GpuTexture>,
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
        }
    }

    fn debug_stats(&self) {
        tracing::debug!("Meshes: {}", self.mesh_data.keys().len());
        tracing::debug!("Transient: {}", self.transient_mesh_data[0].keys().len());
        tracing::debug!("Bindings: {}", self.material_bindings.keys().len());
    }

    pub fn new_frame(&mut self, ctx: &Context) -> usize {
        self.debug_stats();
        let frame_id = ctx.frame_id();

        self.transient_mesh_data[frame_id].clear();
        frame_id
    }

    pub fn add_object(
        &mut self,
        ctx: &Context,
        object: Arc<Model>,
        pass: Arc<dyn RenderPass>,
        transient: bool,
    ) -> &[GPUMesh] {
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
        ctx: &Context,
        bounding_box: BoundingBox,
        pass: Arc<dyn RenderPass>,
    ) -> &[GPUMesh] {
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
        ctx: &Context,
        model: Arc<Model>,
        pass: Arc<dyn RenderPass>,
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
                vertices.append(&mut vertice.raw(vertex_flags, alignment));
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
        pass: Arc<dyn RenderPass>,
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

    fn upload_vertices(
        ctx: &Context,
        vertices: &[u8],
        indices: &[u32],
    ) -> (Arc<GfxBuffer>, Arc<GfxBuffer>) {
        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();

        let mut staging = GfxBuffer::new(
            "staging",
            indices_size + vertices_size,
            BufferUsage::Staging,
            &ctx.device,
        );

        let vertex_buffer =
            GfxBuffer::new("vertex", vertices_size, BufferUsage::Vertex, &ctx.device);
        let index_buffer = GfxBuffer::new("index", indices_size, BufferUsage::Index, &ctx.device);

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