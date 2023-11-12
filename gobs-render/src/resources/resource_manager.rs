use std::collections::HashMap;

use crate::model::{InstanceData, Model, ModelId};
use crate::resources::mesh::MeshData;
use crate::resources::ModelInstance;
use crate::{
    context::Gfx,
    model::{Material, MaterialId, Mesh, MeshId},
    shader::{Shader, ShaderId},
};

pub struct ResourceManager {
    mesh_buffers: HashMap<(MeshId, ShaderId), MeshData>,
    material_bind_groups: HashMap<MaterialId, wgpu::BindGroup>,
    instance_buffers: HashMap<(ModelId, ShaderId), ModelInstance>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            mesh_buffers: HashMap::new(),
            material_bind_groups: HashMap::new(),
            instance_buffers: HashMap::new(),
        }
    }

    pub fn update_instance_data(
        &mut self,
        gfx: &Gfx,
        model: &Model,
        shader: &Shader,
        instances: &Vec<InstanceData>,
    ) {
        let key = (model.id, shader.id);

        if !self.instance_buffers.contains_key(&key) {
            let instance_buffer = gfx.create_instance_buffer(instances, shader.instance_flags);

            self.instance_buffers.insert(
                key,
                ModelInstance {
                    instance_buffer,
                    instance_count: instances.len(),
                },
            );
        } else {
            let model_instance = self.instance_buffers.get_mut(&key).unwrap();

            gfx.update_instance_buffer(
                &model_instance.instance_buffer,
                instances,
                shader.instance_flags,
            );
            model_instance.instance_count = instances.len();
        }
    }

    pub fn instance_data(&self, model: &Model, shader: &Shader) -> &ModelInstance {
        let key = (model.id, shader.id);

        let model_instance = self.instance_buffers.get(&key).unwrap();

        model_instance
    }

    pub fn update_material_bind_group(
        &mut self,
        gfx: &Gfx,
        material: &Material,
        layout: &wgpu::BindGroupLayout,
    ) {
        if !self.material_bind_groups.contains_key(&material.id) {
            let bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &material.diffuse_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&material.diffuse_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&material.normal_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&material.normal_texture.sampler),
                    },
                ],
                label: None,
            });

            self.material_bind_groups.insert(material.id, bind_group);
        }
    }

    pub fn material_bind_group(&self, material: &Material) -> &wgpu::BindGroup {
        self.material_bind_groups.get(&material.id).unwrap()
    }

    pub fn update_mesh_buffer(&mut self, gfx: &Gfx, mesh: &Mesh, shader: &Shader) {
        let key = (mesh.id, shader.id);

        if !self.mesh_buffers.contains_key(&key) {
            let vertex_buffer = gfx.create_vertex_buffer(&mesh.vertices, shader.vertex_flags);
            let index_buffer = gfx.create_index_buffer(&mesh.indices);

            let mesh_data = MeshData {
                vertex_buffer: vertex_buffer,
                index_buffer: index_buffer,
                num_elements: mesh.indices.len(),
            };

            self.mesh_buffers.insert(key, mesh_data);
        };
    }

    pub fn mesh_buffer(&self, mesh: &Mesh, shader: &Shader) -> &MeshData {
        self.mesh_buffers.get(&(mesh.id, shader.id)).unwrap()
    }
}
