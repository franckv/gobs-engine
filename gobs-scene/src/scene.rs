use anyhow::Result;
use glam::Quat;
use glam::Vec3;
use log::*;

use gobs_wgpu as render;

use render::render::RenderError;
use render::shader::Shader;
use render::shader::ShaderBindGroup;
use render::shader::ShaderType;
use render::shader_data::InstanceData;
use uuid::Uuid;

use crate::assets;
use crate::camera::Camera;
use crate::light::Light;
use crate::node::Node;
use render::model::CameraResource;
use render::model::LightResource;
use render::model::{Model, Texture};
use render::render::Batch;
use render::render::Gfx;
use render::shader::{PhongShader, SolidShader};

struct ModelInstance {
    model: Model,
    shader: ShaderType,
    instance_buffer: Option<wgpu::Buffer>,
    instance_count: usize,
}

pub struct Scene {
    pub solid_shader: Shader,
    pub phong_shader: Shader,
    pub camera: Camera,
    pub light: Light,
    pub camera_resource: CameraResource,
    pub light_resource: LightResource,
    depth_texture: Texture,
    pub nodes: Vec<Node>,
    models: Vec<ModelInstance>,
}

impl Scene {
    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub async fn new(gfx: &Gfx, camera: Camera, light: Light) -> Self {
        info!("New scene");

        let solid_shader = SolidShader::new(&gfx).await;
        let phong_shader = PhongShader::new(&gfx).await;

        let camera_resource =
            gfx.create_camera_resource(phong_shader.layout(ShaderBindGroup::Camera));

        let light_resource = gfx.create_light_resource(phong_shader.layout(ShaderBindGroup::Light));

        let models = Vec::new();

        let nodes = Vec::new();

        let depth_texture = Texture::create_depth_texture(gfx, "depth_texture");

        Scene {
            solid_shader,
            phong_shader,
            camera,
            light,
            camera_resource,
            light_resource,
            depth_texture,
            nodes,
            models,
        }
    }

    pub fn resize(&mut self, gfx: &Gfx, width: u32, height: u32) {
        self.depth_texture = Texture::create_depth_texture(gfx, "depth_texture");
        self.camera.projection.resize(width, height);
    }

    pub fn update(&mut self, gfx: &Gfx) {
        let view_position = self.camera.position.extend(1.0).to_array();
        let view_proj =
            (self.camera.projection.to_matrix() * self.camera.to_matrix()).to_cols_array_2d();

        self.camera_resource.update(gfx, view_position, view_proj);

        self.light_resource
            .update(&gfx, self.light.position.into(), self.light.colour.into());

        for model in &mut self.models {
            let instance_data = self
                .nodes
                .iter()
                .filter(|n| n.model() == model.model.id)
                .map(|n| {
                    InstanceData::new(model.shader.instance_flags())
                        .model_transform(
                            n.transform().position,
                            n.transform().rotation,
                            model.model.scale,
                        )
                        .normal_rot(n.transform().rotation)
                        .build()
                })
                .collect::<Vec<_>>();

            match &model.instance_buffer {
                Some(instance_buffer) => {
                    gfx.update_instance_buffer(&instance_buffer, &instance_data);
                }
                None => {
                    model.instance_buffer = Some(gfx.create_instance_buffer(&instance_data));
                }
            }
            model.instance_count = instance_data.len();
        }
    }

    pub fn add_node(&mut self, position: Vec3, rotation: Quat, model: Uuid) {
        let node = Node::new(position, rotation, model);
        self.nodes.push(node);
    }

    pub fn add_model(&mut self, model: Model, shader: ShaderType) -> Uuid {
        let id = model.id;

        let model_instance = ModelInstance {
            model,
            shader,
            instance_buffer: None,
            instance_count: 0,
        };

        self.models.push(model_instance);

        id
    }

    pub async fn load_model(
        &mut self,
        gfx: &Gfx,
        name: &str,
        shader_type: ShaderType,
        scale: f32,
    ) -> Result<Uuid> {
        let shader = match shader_type {
            ShaderType::Phong => &self.phong_shader,
            ShaderType::Solid => &self.solid_shader,
        };
        let model = assets::load_model(name, gfx, shader, scale).await?;

        let id = self.add_model(model, shader_type);

        Ok(id)
    }

    pub fn render(&self, gfx: &Gfx) -> Result<(), RenderError> {
        let mut batch = Batch::begin(gfx)
            .depth_texture(&self.depth_texture)
            .camera_resource(&self.camera_resource)
            .light_resource(&self.light_resource);

        for model in &self.models {
            match model.shader {
                ShaderType::Phong => {
                    batch = batch.draw_indexed(
                        &model.model,
                        &self.phong_shader,
                        model.instance_buffer.as_ref().unwrap(),
                        model.instance_count,
                    );
                }
                ShaderType::Solid => {
                    batch = batch.draw_indexed(
                        &model.model,
                        &self.solid_shader,
                        model.instance_buffer.as_ref().unwrap(),
                        model.instance_count,
                    );
                }
            };
        }

        batch.finish().render()
    }
}
