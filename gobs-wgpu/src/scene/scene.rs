use anyhow::Result;
use glam::{Quat, Vec3};
use log::*;

use gobs_scene as scene;

use scene::camera::Camera;
use scene::light::Light;

use crate::camera::CameraResource;
use crate::light::LightResource;
use crate::model::{Model, Texture};
use crate::render::Gfx;
use crate::resource;
use crate::scene::Node;
use crate::shader::{PhongShader, ShaderType, SolidShader};

const LIGHT: &str = "sphere.obj";

pub struct Scene {
    pub solid_shader: SolidShader,
    pub phong_shader: PhongShader,
    pub camera: Camera,
    pub camera_resource: CameraResource,
    light: Light,
    pub light_resource: LightResource,
    depth_texture: Texture,
    pub light_model: Model,
    pub nodes: Vec<Node>,
    pub models: Vec<Model>,
    pub instance_buffers: Vec<wgpu::Buffer>,
}

impl Scene {
    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub async fn new(gfx: &Gfx, camera: Camera, light: Light) -> Self {
        info!("New scene");

        let solid_shader = SolidShader::new(&gfx).await;
        let phong_shader = PhongShader::new(&gfx).await;

        let camera_resource = gfx.create_camera_resource(&phong_shader.layouts[0]);

        let light_resource = gfx.create_light_resource(&phong_shader.layouts[1]);

        let models = Vec::new();

        let nodes = Vec::new();

        let instance_buffers = Vec::new();

        let depth_texture = Texture::create_depth_texture(gfx, "depth_texture");

        let light_model = resource::load_model(LIGHT, gfx, &phong_shader.layouts[2])
            .await
            .unwrap();

        Scene {
            solid_shader,
            phong_shader,
            camera,
            camera_resource,
            light,
            light_resource,
            depth_texture,
            light_model,
            nodes,
            models,
            instance_buffers,
        }
    }

    pub fn resize(&mut self, gfx: &Gfx, width: u32, height: u32) {
        self.depth_texture = Texture::create_depth_texture(gfx, "depth_texture");
        self.camera.projection.resize(width, height);
    }

    pub fn update(&mut self, gfx: &Gfx, dt: f32) {
        self.camera_resource.update(gfx, &self.camera);

        let old_position: Vec3 = self.light.position;
        let position: Vec3 =
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (60.0 * dt).to_radians())
                * old_position)
                .into();

        self.light.update(position);
        self.light_resource.update(&gfx, &self.light);

        for i in 0..self.models.len() {
            let instance_data = self
                .nodes
                .iter()
                .filter(|n| n.model() == i)
                .map(|n| n.transform().to_raw())
                .collect::<Vec<_>>();
            if self.instance_buffers.len() <= i {
                let instance_buffer = gfx.create_instance_buffer(&instance_data);
                self.instance_buffers.push(instance_buffer);
            } else {
                gfx.update_instance_buffer(&self.instance_buffers[i], &instance_data);
            }
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub async fn load_model(&mut self, gfx: &Gfx, name: &str, ty: ShaderType) -> Result<()> {
        let model = match ty {
            ShaderType::Phong => {
                resource::load_model(name, gfx, &self.phong_shader.layouts[2]).await
            }
            ShaderType::Solid => {
                resource::load_model(name, gfx, &self.phong_shader.layouts[2]).await
            }
        };

        self.models.push(model?);

        Ok(())
    }
}
