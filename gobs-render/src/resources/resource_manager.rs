use std::collections::HashMap;

use crate::camera::{Camera, CameraId};
use crate::light::{Light, LightId};
use crate::model::{InstanceData, Model, ModelId};
use crate::resources::mesh::MeshData;
use crate::resources::ModelInstance;
use crate::resources::{CameraResource, LightResource};
use crate::shader::ShaderBindGroup;
use crate::{
    context::Gfx,
    model::{Material, MaterialId, Mesh, MeshId},
    shader::{Shader, ShaderId},
};

pub struct ResourceManager {
    mesh_buffers: HashMap<(MeshId, ShaderId), MeshData>,
    material_bind_groups: HashMap<MaterialId, wgpu::BindGroup>,
    instance_buffers: HashMap<(ModelId, ShaderId), ModelInstance>,
    light_resources: HashMap<(LightId, ShaderId), LightResource>,
    camera_resources: HashMap<(CameraId, ShaderId), CameraResource>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            mesh_buffers: HashMap::new(),
            material_bind_groups: HashMap::new(),
            instance_buffers: HashMap::new(),
            light_resources: HashMap::new(),
            camera_resources: HashMap::new(),
        }
    }

    pub fn update_light(&mut self, gfx: &Gfx, light: &Light, shader: &Shader) {
        let key = (light.id, shader.id);

        self.light_resources
            .entry(key)
            .or_insert_with(|| gfx.create_light_resource(shader.layout(ShaderBindGroup::Light)));

        let light_resource = self.light_resources.get_mut(&key).unwrap();

        light_resource.update(gfx, light.position.into(), light.colour.into());
    }

    pub fn light(&self, light: &Light, shader: &Shader) -> &LightResource {
        let key = (light.id, shader.id);

        let light_resource = self.light_resources.get(&key).unwrap();

        light_resource
    }

    pub fn update_camera(&mut self, gfx: &Gfx, camera: &Camera, shader: &Shader) {
        let key = (camera.id, shader.id);

        self.camera_resources
            .entry(key)
            .or_insert_with(|| gfx.create_camera_resource(shader.layout(ShaderBindGroup::Camera)));

        let camera_resource = self.camera_resources.get_mut(&key).unwrap();

        let view_position = camera.position.extend(1.).to_array();
        let view_proj = camera.view_proj().to_cols_array_2d();

        camera_resource.update(gfx, view_position, view_proj);
    }

    pub fn camera(&self, camera: &Camera, shader: &Shader) -> &CameraResource {
        let key = (camera.id, shader.id);

        let camera_resource = self.camera_resources.get(&key).unwrap();

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
            let model_instance = ModelInstance::new(gfx, instances, shader.instance_flags);

            self.instance_buffers.insert(key, model_instance);
        } else {
            let model_instance = self.instance_buffers.get_mut(&key).unwrap();

            model_instance.update(gfx, instances, shader.instance_flags);
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
        self.material_bind_groups
            .entry(material.id)
            .or_insert_with(|| {
                gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
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
                            resource: wgpu::BindingResource::Sampler(
                                &material.diffuse_texture.sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                &material.normal_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(
                                &material.normal_texture.sampler,
                            ),
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

            MeshData {
                vertex_buffer,
                index_buffer,
                num_elements: mesh.indices.len(),
            }
        });
    }

    pub fn mesh_buffer(&self, mesh: &Mesh, shader: &Shader) -> &MeshData {
        self.mesh_buffers.get(&(mesh.id, shader.id)).unwrap()
    }
}
