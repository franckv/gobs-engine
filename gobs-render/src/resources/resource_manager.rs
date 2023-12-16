use std::collections::HashMap;

use uuid::Uuid;

use gobs_core as core;
use gobs_material as material;

use core::entity::camera::Camera;
use core::entity::instance::InstanceData;
use core::entity::light::Light;
use core::entity::uniform::{UniformDataBuilder, UniformProp};
use core::geometry::mesh::{Mesh, MeshId};
use material::TextureId;
use material::{Material, MaterialId};

use crate::model::{Model, ModelId};
use crate::resources::mesh::MeshBuffer;
use crate::resources::InstanceBuffer;
use crate::resources::TextureBuffer;
use crate::resources::UniformResource;
use crate::{
    context::Gfx,
    shader::{Shader, ShaderId},
};

pub struct ResourceManager {
    mesh_buffers: HashMap<(MeshId, ShaderId), MeshBuffer>,
    material_bind_groups: HashMap<MaterialId, wgpu::BindGroup>,
    instance_buffers: HashMap<(ModelId, ShaderId), InstanceBuffer>,
    uniform_resources: HashMap<(Uuid, ShaderId), UniformResource>,
    texture_resources: HashMap<TextureId, TextureBuffer>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            mesh_buffers: HashMap::new(),
            material_bind_groups: HashMap::new(),
            instance_buffers: HashMap::new(),
            uniform_resources: HashMap::new(),
            texture_resources: HashMap::new(),
        }
    }

    pub fn update_light(&mut self, gfx: &Gfx, light: &Light, shader: &Shader) {
        let key = (light.id, shader.id);

        let position = UniformProp::Vec3F(light.position.into());
        let colour = UniformProp::Vec3F(light.colour.into());

        self.uniform_resources.entry(key).or_insert_with(|| {
            let light_data = UniformDataBuilder::new("light")
                .prop("position", position)
                .prop("colour", colour)
                .build();

            UniformResource::new(gfx, light_data)
        });

        let light_resource = self.uniform_resources.get_mut(&key).unwrap();

        light_resource.data.update("position", position);
        light_resource.data.update("colour", colour);

        light_resource.update(gfx);
    }

    pub fn light(&self, light: &Light, shader: &Shader) -> &UniformResource {
        let key = (light.id, shader.id);

        let light_resource = self.uniform_resources.get(&key).unwrap();

        light_resource
    }

    pub fn update_camera(&mut self, gfx: &Gfx, camera: &Camera, shader: &Shader) {
        let key = (camera.id, shader.id);
        let view_position = UniformProp::Vec4F(camera.position.extend(1.).into());
        let view_proj = UniformProp::Mat4F(camera.view_proj().to_cols_array_2d());

        self.uniform_resources.entry(key).or_insert_with(|| {
            let camera_data = UniformDataBuilder::new("camera")
                .prop("view_position", view_position)
                .prop("view_proj", view_proj)
                .build();

            UniformResource::new(gfx, camera_data)
        });

        let camera_resource = self.uniform_resources.get_mut(&key).unwrap();

        camera_resource.data.update("view_position", view_position);
        camera_resource.data.update("view_proj", view_proj);

        camera_resource.update(gfx);
    }

    pub fn camera(&self, camera: &Camera, shader: &Shader) -> &UniformResource {
        let key = (camera.id, shader.id);

        let camera_resource = self.uniform_resources.get(&key).unwrap();

        camera_resource
    }

    pub fn update_instance_data(
        &mut self,
        gfx: &Gfx,
        model: &Model,
        shader: &Shader,
        instances: &[InstanceData],
    ) {
        let key = (model.id, shader.id);

        if !self.instance_buffers.contains_key(&key) {
            let model_instance = InstanceBuffer::new(gfx, instances, shader.instance_flags);

            self.instance_buffers.insert(key, model_instance);
        } else {
            let model_instance = self.instance_buffers.get_mut(&key).unwrap();

            model_instance.update(gfx, instances, shader.instance_flags);
        }
    }

    pub fn instance_data(&self, model: &Model, shader: &Shader) -> &InstanceBuffer {
        let key = (model.id, shader.id);

        let model_instance = self.instance_buffers.get(&key).unwrap();

        model_instance
    }

    pub fn update_material(
        &mut self,
        gfx: &Gfx,
        material: &Material,
        layout: &wgpu::BindGroupLayout,
    ) {
        let diffuse_texture_id = material.diffuse_texture.read().unwrap().id;

        if material.diffuse_texture.read().unwrap().dirty {
            log::debug!("Replace texture {}", diffuse_texture_id);
            material.diffuse_texture.write().unwrap().dirty = false;
            self.texture_resources.remove(&diffuse_texture_id);
            self.material_bind_groups.remove(&material.id);
        }

        self.texture_resources
            .entry(diffuse_texture_id)
            .or_insert_with(|| {
                log::debug!("Insert texture {}", diffuse_texture_id);
                TextureBuffer::new(gfx, material.diffuse_texture.read().unwrap().clone())
            });

        let normal_texture_id = material.normal_texture.read().unwrap().id;

        if material.normal_texture.read().unwrap().dirty {
            log::debug!("Replace normal texture {}", diffuse_texture_id);
            material.normal_texture.write().unwrap().dirty = false;
            self.texture_resources.remove(&normal_texture_id);
            self.material_bind_groups.remove(&material.id);
        }

        self.texture_resources
            .entry(normal_texture_id)
            .or_insert_with(|| {
                log::debug!("Insert normal texture {}", diffuse_texture_id);
                TextureBuffer::new(gfx, material.normal_texture.read().unwrap().clone())
            });

        let diffuse_texture = self
            .texture_resources
            .get(&material.diffuse_texture.read().unwrap().id)
            .unwrap();

        let normal_texture = self
            .texture_resources
            .get(&material.normal_texture.read().unwrap().id)
            .unwrap();

        self.material_bind_groups
            .entry(material.id)
            .or_insert_with(|| {
                gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                        },
                    ],
                    label: None,
                })
            });
    }

    pub fn material_bind_group(&self, material: &Material) -> &wgpu::BindGroup {
        self.material_bind_groups.get(&material.id).unwrap()
    }

    pub fn update_mesh_buffer(&mut self, gfx: &Gfx, mesh: &Mesh, shader: &Shader) {
        let key = (mesh.id, shader.id);

        self.mesh_buffers.entry(key).or_insert_with(|| {
            let vertex_buffer = gfx.create_vertex_buffer(&mesh.vertices, shader.vertex_flags);
            let index_buffer = gfx.create_index_buffer(&mesh.indices);

            MeshBuffer {
                vertex_buffer,
                index_buffer,
                num_elements: mesh.indices.len(),
            }
        });
    }

    pub fn mesh_buffer(&self, mesh: &Mesh, shader: &Shader) -> &MeshBuffer {
        self.mesh_buffers.get(&(mesh.id, shader.id)).unwrap()
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
