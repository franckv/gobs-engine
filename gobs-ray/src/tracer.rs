use std::sync::Arc;

use glam::{Quat, Vec3};

use gobs_core::{
    entity::{camera::Camera, instance::InstanceFlag, light::Light},
    geometry::vertex::VertexFlag,
    Color,
};
use gobs_render::{context::Gfx, pipeline::PipelineFlag, shader::Shader};
use gobs_scene::{
    shape::Shapes, Material, MaterialBuilder, Model, ModelBuilder, RenderError, Scene,
};

use crate::Ray;

pub struct Tracer {
    width: u32,
    height: u32,
    scene: Scene,
    camera: Camera,
    material: Arc<Material>,
    shader: Arc<Shader>,
    framebuffer: Vec<Color>,
    background: fn(Ray) -> Color,
    changed: bool,
}

impl Tracer {
    const LAYER: &'static str = "tracer";
    const SHADER: &'static str = "ui.wgsl";

    pub async fn new(gfx: &Gfx, width: u32, height: u32, background: fn(Ray) -> Color) -> Self {
        let light = Light::new((0., 0., 10.), Color::WHITE);

        let frame_camera = Camera::ortho(
            (0., 0., 1.),
            width as f32,
            height as f32,
            0.1,
            100.,
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        let mut scene = Scene::new(gfx, frame_camera, light, &[]).await;

        let shader = Shader::new(
            gfx,
            "shader",
            Self::SHADER,
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE,
            InstanceFlag::MODEL,
            PipelineFlag::ALPHA,
        )
        .await;

        let framebuffer = vec![];

        let material = MaterialBuilder::new("diffuse")
            .diffuse_buffer(&framebuffer, width as u32, height as u32)
            .await
            .build();

        let image: Arc<Model> = ModelBuilder::new()
            .add_mesh(Shapes::quad(), Some(material.clone()))
            .build(shader.clone());

        scene.add_node(
            Self::LAYER,
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [width as f32, height as f32, 1.].into(),
            image,
        );

        Tracer {
            width,
            height,
            scene,
            camera: Camera::perspective(
                Vec3::ZERO,
                width as f32 / height as f32,
                (45. as f32).to_radians(),
                0.1,
                100.,
                (-90. as f32).to_radians(),
                (0. as f32).to_radians(),
                Vec3::Y,
            ),
            material,
            shader,
            framebuffer,
            background,
            changed: true,
        }
    }

    fn update_buffer(&mut self) {
        for i in 0..(self.height as u32) {
            for j in 0..(self.width as u32) {
                // -2..2
                let x = -2. + 4. * (j as f32 / self.width as f32);
                // -1..1
                let y = 1. - 2. * (i as f32 / self.height as f32);

                let ray = Ray {
                    origin: self.camera.position,
                    direction: Vec3::new(x, y, 1.).normalize(),
                };

                let bg: fn(Ray) -> Color = self.background;
                self.framebuffer.push(bg(ray));
            }
        }
    }

    pub fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    pub fn update(&mut self, gfx: &Gfx) {
        if self.changed {
            self.update_buffer();

            self.scene.layer_mut(Self::LAYER).nodes_mut().clear();
            let data = self
                .framebuffer
                .iter()
                .flat_map(|c| Into::<[u8; 4]>::into(*c))
                .collect::<Vec<u8>>();

            self.material
                .diffuse_texture
                .write()
                .unwrap()
                .update_texture(data);

            let image: Arc<Model> = ModelBuilder::new()
                .add_mesh(Shapes::quad(), Some(self.material.clone()))
                .build(self.shader.clone());

            self.scene.add_node(
                Self::LAYER,
                [0., 0., 0.].into(),
                Quat::IDENTITY,
                [self.width as f32, self.height as f32, 1.].into(),
                image,
            );
        }

        self.changed = false;
        self.scene.update(gfx);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.scene.resize(width, height);
    }
}
