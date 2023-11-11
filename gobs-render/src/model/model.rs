use std::sync::Arc;

use uuid::Uuid;

use crate::{
    model::{Material, Mesh, VertexFlag},
    render::Gfx,
    shader::{Shader, ShaderBindGroup},
};

pub struct ModelBuilder {
    meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder { meshes: Vec::new() }
    }

    pub fn meshes(mut self, meshes: Vec<(Arc<Mesh>, Option<Arc<Material>>)>) -> Self {
        self.meshes = meshes;

        self
    }

    pub fn add_mesh(mut self, mesh: Arc<Mesh>, material: Option<Arc<Material>>) -> Self {
        self.meshes.push((mesh, material));

        self
    }

    pub fn build(self, gfx: &Gfx, shader: Arc<Shader>) -> Arc<Model> {
        let mesh_data = self
            .meshes
            .iter()
            .map(|(mesh, material)| {
                let bind_group = match material {
                    Some(material) => {
                        if shader.vertex_flags.contains(VertexFlag::TEXTURE) {
                            Some(material.bind_group(gfx, shader.layout(ShaderBindGroup::Material)))
                        } else {
                            None
                        }
                    }
                    None => None,
                };
                let buffers = mesh.create_buffers(gfx, shader.clone());

                MeshData {
                    vertex_buffer: buffers.0,
                    index_buffer: buffers.1,
                    num_elements: mesh.indices.len(),
                    bind_group,
                }
            })
            .collect();

        Arc::new(Model {
            id: Uuid::new_v4(),
            shader,
            mesh_data,
        })
    }
}

pub struct MeshData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: usize,
    pub bind_group: Option<wgpu::BindGroup>,
}

pub struct Model {
    pub id: Uuid,
    pub shader: Arc<Shader>,
    pub mesh_data: Vec<MeshData>,
}