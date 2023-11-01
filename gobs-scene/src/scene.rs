use std::sync::Arc;

use anyhow::Result;
use glam::Quat;
use glam::Vec3;
use log::*;

use gobs_wgpu as render;

use render::model::{
    InstanceData, InstanceFlag, Model, ModelInstance, Texture, TextureType, VertexFlag,
};
use render::pipeline::PipelineFlag;
use render::render::{Batch, Gfx, RenderError};
use render::resources::{CameraResource, LightResource};
use render::shader::{Shader, ShaderBindGroup};

use crate::assets;
use crate::camera::Camera;
use crate::light::Light;
use crate::node::Node;

pub struct Scene {
    pub camera: Camera,
    pub light: Light,
    pub camera_resource: CameraResource,
    pub light_resource: LightResource,
    pub phong_shader: Arc<Shader>,
    pub solid_shader: Arc<Shader>,
    pub ui_shader: Arc<Shader>,
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

        let phong_shader = Shader::new(
            gfx,
            "Phong",
            "phong.wgsl",
            VertexFlag::POSITION | VertexFlag::TEXTURE | VertexFlag::NORMAL,
            InstanceFlag::MODEL | InstanceFlag::NORMAL,
            PipelineFlag::CULLING | PipelineFlag::DEPTH,
        )
        .await;

        let solid_shader = Shader::new(
            gfx,
            "Solid",
            "solid.wgsl",
            VertexFlag::POSITION | VertexFlag::COLOR,
            InstanceFlag::MODEL,
            PipelineFlag::CULLING | PipelineFlag::DEPTH,
        )
        .await;

        let ui_shader = Shader::new(
            gfx,
            "UI",
            "ui.wgsl",
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE,
            InstanceFlag::MODEL,
            PipelineFlag::empty(),
        )
        .await;

        let camera_resource =
            gfx.create_camera_resource(phong_shader.layout(ShaderBindGroup::Camera));

        let light_resource = gfx.create_light_resource(phong_shader.layout(ShaderBindGroup::Light));

        let models = Vec::new();

        let nodes = Vec::new();

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
            phong_shader,
            solid_shader,
            ui_shader,
            depth_texture,
            nodes,
            models,
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
            .update(&gfx, self.light.position.into(), self.light.colour.into());

        for model in &mut self.models {
            let instance_data = self
                .nodes
                .iter()
                .filter(|n| n.model().id == model.model.id)
                .map(|n| {
                    InstanceData::new(model.model.shader.instance_flags)
                        .model_transform(
                            n.transform().translation,
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

    pub fn add_node(&mut self, position: Vec3, rotation: Quat, model: Arc<Model>) {
        let exist = self.models.iter().find(|m| m.model.id == model.id);

        if exist.is_none() {
            let model_instance = ModelInstance {
                model: model.clone(),
                instance_buffer: None,
                instance_count: 0,
            };

            self.models.push(model_instance);
        };
        let node = Node::new(position, rotation, model);
        self.nodes.push(node);
    }

    pub async fn load_model(
        &mut self,
        gfx: &Gfx,
        name: &str,
        shader: Arc<Shader>,
        scale: Vec3,
    ) -> Result<Arc<Model>> {
        let model = assets::load_model(name, gfx, shader.clone(), scale).await?;

        Ok(model)
    }

    pub fn render(&self, gfx: &Gfx) -> Result<(), RenderError> {
        let mut batch = Batch::begin(gfx)
            .depth_texture(&self.depth_texture)
            .camera_resource(&self.camera_resource)
            .light_resource(&self.light_resource);

        for instance in &self.models {
            batch = batch.draw_indexed(
                &instance.model,
                instance.instance_buffer.as_ref().unwrap(),
                instance.instance_count,
            );
        }

        batch.finish().render()
    }
}
