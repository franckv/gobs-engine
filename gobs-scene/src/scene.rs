use anyhow::Result;
use glam::{Quat, Vec3};
use log::*;

use gobs_wgpu as render;
use render::render::RenderError;

use crate::camera::Camera;
use crate::light::Light;
use crate::node::Node;
use render::model::CameraResource;
use render::model::InstanceRaw;
use render::model::LightResource;
use render::model::{Model, Texture};
use render::render::Gfx;
use render::resource;
use render::shader::{PhongShader, SolidShader};

const LIGHT: &str = "sphere.obj";

pub struct Scene {
    pub solid_shader: SolidShader,
    pub phong_shader: PhongShader,
    pub camera: Camera,
    pub light: Light,
    pub camera_resource: CameraResource,
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
            light,
            camera_resource,
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
        let view_position = self.camera.position.extend(1.0).to_array();
        let view_proj =
            (self.camera.projection.to_matrix() * self.camera.to_matrix()).to_cols_array_2d();

        self.camera_resource.update(gfx, view_position, view_proj);

        let old_position: Vec3 = self.light.position;
        let position: Vec3 =
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (60.0 * dt).to_radians())
                * old_position)
                .into();

        self.light.update(position);
        self.light_resource
            .update(&gfx, self.light.position.into(), self.light.colour.into());

        for i in 0..self.models.len() {
            let instance_data = self
                .nodes
                .iter()
                .filter(|n| n.model() == i)
                .map(|n| InstanceRaw::new(n.transform().position, n.transform().rotation))
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

    pub async fn load_model(&mut self, gfx: &Gfx, name: &str) -> Result<()> {
        let model = resource::load_model(name, gfx, &self.phong_shader.layouts[2]).await;

        self.models.push(model?);

        Ok(())
    }

    pub fn render(&self, gfx: &Gfx) -> Result<(), RenderError> {
        let instance_count = (0..self.models.len()).map(|i| self.nodes.iter().filter(|n| n.model() == i).count()).collect();
        
        gfx.render(
            &self.depth_texture,
            &self.camera_resource,
            &self.light_resource,
            &self.light_model,
            &self.solid_shader,
            &self.phong_shader,
            &self.models,
            &self.instance_buffers,
            &instance_count
        )
    }
}
