use std::sync::Arc;

use glam::{Quat, Vec3};
use rand::Rng;

use gobs_core::{
    entity::{camera::Camera, instance::InstanceFlag, light::Light},
    geometry::vertex::VertexFlag,
    Color,
};
use gobs_render::{context::Gfx, pipeline::PipelineFlag, shader::Shader};
use gobs_scene::{
    shape::Shapes, Material, MaterialBuilder, Model, ModelBuilder, RenderError, Scene,
};
use gobs_utils::timer::Timer;

use crate::{hit::Hitable, Ray};

pub struct Tracer {
    width: u32,
    height: u32,
    scene: Scene,
    models: Vec<Box<dyn Hitable>>,
    camera: Camera,
    material: Arc<Material>,
    shader: Arc<Shader>,
    framebuffer: Vec<Color>,
    background: fn(&Ray) -> Color,
    changed: bool,
    rand_index: usize,
    rand_pool: Vec<f32>,
}

impl Tracer {
    const LAYER: &'static str = "tracer";
    const SHADER: &'static str = "ui.wgsl";
    const N_RAYS: u32 = 10;
    const RNG_MAX: usize = 5000;

    pub async fn new(gfx: &Gfx, width: u32, height: u32, background: fn(&Ray) -> Color) -> Self {
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
            PipelineFlag::empty(),
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

        let mut rng = rand::thread_rng();

        Tracer {
            width,
            height,
            scene,
            models: Vec::new(),
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
            rand_index: 0,
            rand_pool: (0..Self::RNG_MAX)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect::<Vec<f32>>(),
        }
    }

    pub fn add_model(&mut self, model: Box<dyn Hitable>) {
        self.models.push(model);

        self.changed = true;
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

    fn next_rng(&mut self) -> f32 {
        let r = self.rand_pool[self.rand_index];
        self.rand_index = (self.rand_index + 1) % Self::RNG_MAX;

        r
    }

    fn update_buffer(&mut self) {
        let mut timer = Timer::new();
        let mut total_rays = 0;

        self.framebuffer.clear();

        for i in 0..(self.height as u32) {
            for j in 0..(self.width as u32) {
                let mut c = Color::BLACK;
                for _ in 0..Self::N_RAYS {
                    // -2..2
                    let x = -2. + 4. * ((j as f32 + self.next_rng()) / self.width as f32);
                    // -1..1
                    let y = 1. - 2. * ((i as f32 + self.next_rng()) / self.height as f32);

                    total_rays += 1;
                    let ray = Ray {
                        origin: self.camera.position,
                        direction: Vec3::new(x, y, 1.).normalize(),
                    };

                    let bg: fn(&Ray) -> Color = self.background;

                    let hit = self
                        .models
                        .iter()
                        .filter_map(|m| m.hit(&ray, 0.1, 100.))
                        .min_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap());

                    match hit {
                        Some(hit) => c = c + hit.color,
                        None => c = c + bg(&ray),
                    }
                }

                c = c / Self::N_RAYS as f32;

                self.framebuffer.push(c);
            }
        }

        log::info!("Rendering time: {} rays in {}", total_rays, timer.delta());
    }
}
