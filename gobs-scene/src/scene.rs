use std::sync::Arc;

use anyhow::Result;
use glam::{Quat, Vec3};
use log::*;

use gobs_render as render;

use render::model::{Model, Texture, TextureType};
use render::render::{Batch, Gfx, RenderError};
use render::resources::{CameraResource, LightResource};
use render::shader::{Shader, ShaderBindGroup};

use crate::assets;
use crate::camera::Camera;
use crate::layer::Layer;
use crate::light::Light;

pub struct Scene {
    pub camera: Camera,
    pub light: Light,
    pub camera_resource: CameraResource,
    pub light_resource: LightResource,
    depth_texture: Texture,
    layers: Vec<Layer>,
}

impl Scene {
    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn layer_mut(&mut self, layer_name: &str) -> &mut Layer {
        let exists = self.layers.iter().find(|l| l.name.eq(layer_name)).is_some();

        if exists {
            self.layers
                .iter_mut()
                .find(|l| l.name.eq(layer_name))
                .expect("Layer exists")
        } else {
            self.layers.push(Layer::new(layer_name));
            self.layers.last_mut().unwrap()
        }
    }

    pub async fn new(gfx: &Gfx, camera: Camera, light: Light, default_shader: Arc<Shader>) -> Self {
        info!("New scene");

        let camera_resource =
            gfx.create_camera_resource(default_shader.layout(ShaderBindGroup::Camera));

        let light_resource =
            gfx.create_light_resource(default_shader.layout(ShaderBindGroup::Light));

        let layers = Vec::new();

        let depth_texture = Texture::new(
            gfx,
            "depth_texture",
            TextureType::DEPTH,
            gfx.width(),
            gfx.height(),
            &[],
        );

        Scene {
            camera,
            light,
            camera_resource,
            light_resource,
            depth_texture,
            layers,
        }
    }

    pub fn resize(&mut self, gfx: &Gfx, width: u32, height: u32) {
        self.depth_texture = Texture::new(
            gfx,
            "depth_texture",
            TextureType::DEPTH,
            gfx.width(),
            gfx.height(),
            &[],
        );
        self.camera.resize(width, height);
    }

    pub fn update(&mut self, gfx: &Gfx) {
        let view_position = self.camera.position.extend(1.).to_array();
        let view_proj = self.camera.view_proj().to_cols_array_2d();

        self.camera_resource.update(gfx, view_position, view_proj);

        self.light_resource
            .update(gfx, self.light.position.into(), self.light.colour.into());

        for layer in &mut self.layers {
            if layer.visible {
                layer.update(gfx);
            }
        }
    }

    pub async fn load_model(
        &mut self,
        gfx: &Gfx,
        name: &str,
        shader: Arc<Shader>,
    ) -> Result<Arc<Model>> {
        let model = assets::load_model(name, gfx, shader.clone()).await?;

        Ok(model)
    }

    pub fn toggle_layer(&mut self, layer_name: &str) {
        self.layer_mut(layer_name).visible = !self.layer_mut(layer_name).visible;
    }

    pub fn add_node(
        &mut self,
        layer_name: &str,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
        model: Arc<Model>,
    ) {
        let layer = self.layer_mut(layer_name);
        layer.add_node(position, rotation, scale, model.clone());
    }

    pub fn render(&self, gfx: &Gfx) -> Result<(), RenderError> {
        let mut batch = Batch::begin(gfx)
            .depth_texture(&self.depth_texture)
            .camera_resource(&self.camera_resource)
            .light_resource(&self.light_resource);

        for layer in &self.layers {
            if layer.visible {
                batch = layer.render(batch);
            }
        }

        batch.finish().render()
    }
}
