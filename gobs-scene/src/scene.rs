use std::sync::Arc;

use anyhow::Result;
use glam::{Quat, Vec3};
use log::*;

use gobs_render as render;

use render::context::Gfx;
use render::graph::batch::Batch;
use render::graph::graph::{RenderError, RenderGraph};
use render::model::{Material, Model};
use render::resources::{CameraResource, LightResource};
use render::shader::{Shader, ShaderBindGroup};

use crate::assets;
use crate::camera::Camera;
use crate::layer::Layer;
use crate::light::Light;

pub struct Scene {
    pub render_graph: RenderGraph,
    pub camera: Camera,
    pub light: Light,
    pub camera_resource: CameraResource,
    pub light_resource: LightResource,
    layers: Vec<Layer>,
}

impl Scene {
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

    pub async fn new(
        gfx: &Gfx,
        camera: Camera,
        light: Light,
        default_shader: Arc<Shader>,
        shaders: &[Arc<Shader>],
    ) -> Self {
        info!("New scene");

        let camera_resource =
            gfx.create_camera_resource(default_shader.layout(ShaderBindGroup::Camera));

        let light_resource =
            gfx.create_light_resource(default_shader.layout(ShaderBindGroup::Light));

        let layers = Vec::new();

        let render_graph = RenderGraph::new("graph", gfx, shaders);

        Scene {
            render_graph,
            camera,
            light,
            camera_resource,
            light_resource,
            layers,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);

        for layer in &mut self.layers {
            layer.resize(width, height);
        }
    }

    pub fn toggle_pass(&mut self, pass_name: &str) {
        self.render_graph.toggle_pass(pass_name);
    }

    pub fn update(&mut self, gfx: &Gfx) {
        let view_position = self.camera.position.extend(1.).to_array();
        let view_proj = self.camera.view_proj().to_cols_array_2d();

        self.camera_resource.update(gfx, view_position, view_proj);

        self.light_resource
            .update(gfx, self.light.position.into(), self.light.colour.into());

        for layer in &mut self.layers {
            if layer.visible() {
                layer.update(gfx);
            }
        }
    }

    pub async fn load_model(
        &mut self,
        gfx: &Gfx,
        name: &str,
        default_material: Option<Arc<Material>>,
        shader: Arc<Shader>,
    ) -> Result<Arc<Model>> {
        let model = assets::load_model(name, gfx, default_material, shader.clone()).await?;

        Ok(model)
    }

    pub fn toggle_layer(&mut self, layer_name: &str) {
        self.layer_mut(layer_name).toggle();
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

    pub fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        let mut batch = Batch::begin()
            .camera_resource(&self.camera_resource)
            .light_resource(&self.light_resource);

        for layer in &self.layers {
            if layer.visible() {
                batch = layer.render(batch);
            }
        }

        self.render_graph.execute(gfx, batch.finish())
    }
}
