use std::sync::Arc;

use glam::{Quat, Vec3};
use rand::{seq::SliceRandom, Rng};

use crate::{hit::Hitable, Ray};
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

struct RngPool {
    index: usize,
    pool: Vec<f32>,
}

impl RngPool {
    const RNG_MAX: usize = 5000;

    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            index: 0,
            pool: (0..Self::RNG_MAX)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect::<Vec<f32>>(),
        }
    }

    pub fn next(&mut self) -> f32 {
        let r = self.pool[self.index];
        self.index = (self.index + 1) % Self::RNG_MAX;

        r
    }
}

pub struct Tracer {
    width: u32,
    height: u32,
    scene: Scene,
    models: Vec<Box<dyn Hitable>>,
    camera: Camera,
    material: Arc<Material>,
    shader: Arc<Shader>,
    framebuffer: Vec<Color>,
    n_rays: u32,
    background: fn(&Ray) -> Color,
    changed: bool,
    draw_indexes: Vec<usize>,
    rng: RngPool,
    timer: Timer,
}

impl Tracer {
    const LAYER: &'static str = "tracer";
    const SHADER: &'static str = "ui.wgsl";
    const PIXEL_PER_FRAME: usize = 20000;
    const MAX_REFLECT: u32 = 10;
    const MIN_DISTANCE: f32 = 0.1;
    const MAX_DISTANCE: f32 = 200.;

    pub async fn new(
        gfx: &Gfx,
        width: u32,
        height: u32,
        camera: Camera,
        n_rays: u32,
        background: fn(&Ray) -> Color,
    ) -> Self {
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

        let framebuffer = Vec::new();
        let draw_indexes = Vec::new();

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
            models: Vec::new(),
            camera,
            material,
            shader,
            framebuffer,
            n_rays,
            background,
            changed: true,
            draw_indexes,
            rng: RngPool::new(),
            timer: Timer::new(),
        }
    }

    pub fn reset(&mut self) {
        self.framebuffer.clear();
        self.draw_indexes.clear();

        for i in 0..(self.height as usize * self.width as usize) {
            self.framebuffer.push(Color::BLACK);
            self.draw_indexes.push(i);
        }

        let mut rng = rand::thread_rng();
        self.draw_indexes.shuffle(&mut rng)
    }

    pub fn add_model(&mut self, model: Box<dyn Hitable>) {
        self.models.push(model);
        self.reset();
    }

    pub fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    pub fn update(&mut self, gfx: &Gfx) {
        if self.changed {
            self.reset();
            self.timer.reset();
        }

        if self.draw_indexes.len() > 0 {
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

            if self.draw_indexes.is_empty() {
                log::info!("Rendering time: {}", self.timer.delta());
            }
        }

        self.changed = false;
        self.scene.update(gfx);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn update_buffer(&mut self) {
        let indices = self
            .draw_indexes
            .drain(0..Self::PIXEL_PER_FRAME.min(self.draw_indexes.len()))
            .collect::<Vec<usize>>();

        for idx in indices {
            let c = self.compute_pixel(idx);

            self.framebuffer[idx] = c;
        }
    }

    fn compute_pixel(&mut self, idx: usize) -> Color {
        let i = idx / self.width as usize;
        let j = idx % self.width as usize;

        let mut c = Color::BLACK;
        for _ in 0..self.n_rays {
            // -2..2
            let x = -2. + 4. * ((j as f32 + self.rng.next()) / self.width as f32);
            // -1..1
            let y = 1. - 2. * ((i as f32 + self.rng.next()) / self.height as f32);

            let ray = Ray::new(self.camera.position, Vec3::new(x, y, 1.));

            c = c + self.cast(&ray, Self::MAX_REFLECT);
        }

        c = c / self.n_rays as f32;

        c
    }

    fn cast(&self, ray: &Ray, limit: u32) -> Color {
        if limit <= 0 {
            return Color::BLACK;
        }

        let bg: fn(&Ray) -> Color = self.background;

        let hit = self
            .models
            .iter()
            .filter_map(|m| m.hit(&ray, Self::MIN_DISTANCE, Self::MAX_DISTANCE))
            .min_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap());

        match hit {
            Some(hit) => {
                let reflect_color = self.cast(&ray.reflect(hit.position, hit.normal), limit - 1);
                hit.color * (1. - hit.reflect) + reflect_color * hit.reflect
            }
            None => bg(&ray),
        }
    }
}
