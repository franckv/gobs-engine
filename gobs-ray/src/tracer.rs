use std::sync::Arc;

use glam::{Quat, Vec3};
use rayon::prelude::*;

use crate::{
    buffer::{ChunkStrategy, ImageBuffer},
    hit::Hitable,
    Ray,
};
use gobs_core::{
    entity::{camera::Camera, instance::InstanceFlag, light::Light},
    geometry::vertex::VertexFlag,
    Color,
};
use gobs_material::{Material, MaterialBuilder};
use gobs_render::{context::Gfx, pipeline::PipelineFlag, shader::Shader};
use gobs_scene::{shape::Shapes, Model, ModelBuilder, RenderError, Scene};
use gobs_utils::{rng::RngPool, timer::Timer};

pub struct Tracer {
    scene: Scene,
    image_buffer: ImageBuffer,
    models: Vec<Box<dyn Hitable + Sync + Send>>,
    lights: Vec<Light>,
    camera: Camera,
    material: Arc<Material>,
    shader: Arc<Shader>,
    background: fn(&Ray) -> Color,
    n_rays: u32,
    n_reflects: u32,
    n_threads: u32,
    changed: bool,
    timer: Timer,
}

impl Tracer {
    const LAYER: &'static str = "tracer";

    pub fn width(&self) -> u32 {
        self.image_buffer.width
    }

    pub fn height(&self) -> u32 {
        self.image_buffer.height
    }

    pub fn framebuffer(&self) -> &[Color] {
        &self.image_buffer.framebuffer
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.image_buffer.bytes()
    }

    pub fn reset(&mut self) {
        self.image_buffer.reset();
    }

    pub fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    pub fn update(&mut self, gfx: &Gfx) {
        if self.changed {
            self.reset();
            self.timer.reset();
        }

        if !self.image_buffer.is_complete() {
            self.update_buffer();

            self.scene.layer_mut(Self::LAYER).nodes_mut().clear();

            let data = self.image_buffer.bytes();

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
                [
                    self.image_buffer.width as f32,
                    self.image_buffer.height as f32,
                    1.,
                ]
                .into(),
                image,
            );

            if self.image_buffer.is_complete() {
                log::info!("Rendering time: {:.2}s", self.timer.delta());
            }
        }

        self.changed = false;
        self.scene.update(gfx);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn update_buffer(&mut self) {
        let chunks: Vec<Vec<usize>> = (0..self.n_threads)
            .filter_map(|_| match self.image_buffer.is_complete() {
                true => None,
                false => Some(self.image_buffer.get_chunk()),
            })
            .collect();

        let results: Vec<Vec<(usize, Color)>> = if self.n_threads > 1 {
            chunks
                .par_iter()
                .map(|chunk| self.compute_chunk(&chunk))
                .collect()
        } else {
            chunks
                .iter()
                .map(|chunk| self.compute_chunk(&chunk))
                .collect()
        };

        for result in results {
            for (idx, c) in result {
                self.image_buffer.update_pixel(idx, c);
            }
        }
    }

    pub fn compute_chunk(&self, chunk: &[usize]) -> Vec<(usize, Color)> {
        let mut result = Vec::new();

        let mut rng = RngPool::new(chunk.len());

        for idx in chunk {
            let c = self.compute_pixel(*idx, &mut rng);

            result.push((*idx, c));
        }

        result
    }

    fn compute_pixel(&self, idx: usize, rng: &mut RngPool) -> Color {
        let i = idx / self.image_buffer.width as usize;
        let j = idx % self.image_buffer.width as usize;

        let mut c = Color::BLACK;
        for _ in 0..self.n_rays {
            // -2..2
            let x = -2. + 4. * ((j as f32 + rng.next()) / self.image_buffer.width as f32);
            // -1..1
            let y = 1. - 2. * ((i as f32 + rng.next()) / self.image_buffer.height as f32);

            let ray = Ray::new(self.camera.position, Vec3::new(x, y, 1.));

            c = c + self.cast(&ray, self.n_reflects);
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
            .filter_map(|m| m.hit(&ray, self.camera.mode.near(), self.camera.mode.far()))
            .min_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap());

        match hit {
            Some(hit) => {
                let reflect_color = self.cast(&ray.reflect(hit.position, hit.normal), limit - 1);

                for light in &self.lights {
                    let light_direction = light.position - hit.position;
                    let light_ray = Ray::new(hit.position, light_direction);
                    let blocker = self.models.iter().find(|m| {
                        m.hit_distance(&light_ray, self.camera.mode.near(), self.camera.mode.far())
                            .is_some()
                    });

                    if blocker.is_none() {
                        return hit.color * (1. - hit.reflect) + reflect_color * hit.reflect;
                    }
                }

                (hit.color * (1. - hit.reflect) + reflect_color * hit.reflect) * 0.5
            }
            None => bg(&ray),
        }
    }
}

pub struct TracerBuilder {
    width: u32,
    height: u32,
    scene: Scene,
    models: Vec<Box<dyn Hitable + Sync + Send>>,
    lights: Vec<Light>,
    camera: Camera,
    shader: Arc<Shader>,
    background: fn(&Ray) -> Color,
    n_rays: u32,
    n_reflects: u32,
    n_threads: u32,
    strategy: ChunkStrategy,
}

impl TracerBuilder {
    const SHADER: &'static str = "ui.wgsl";

    pub fn default_background(_: &Ray) -> Color {
        Color::BLACK
    }

    pub async fn new(gfx: &Gfx, width: u32, height: u32) -> Self {
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

        let camera = Camera::perspective(
            Vec3::new(0., 0.2, 0.),
            width as f32 / height as f32,
            (45. as f32).to_radians(),
            0.1,
            100.,
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        let scene = Scene::new(gfx, frame_camera, light, &[]).await;

        let shader = Shader::new(
            gfx,
            "shader",
            Self::SHADER,
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE,
            InstanceFlag::MODEL,
            PipelineFlag::empty(),
        )
        .await;

        TracerBuilder {
            width,
            height,
            scene,
            models: Vec::new(),
            lights: Vec::new(),
            camera,
            shader,
            background: Self::default_background,
            n_rays: 10,
            n_reflects: 10,
            n_threads: 1,
            strategy: ChunkStrategy::BOX,
        }
    }

    pub fn background(mut self, background: fn(&Ray) -> Color) -> Self {
        self.background = background;

        self
    }

    pub fn camera(mut self, camera: Camera) -> Self {
        self.camera = camera;

        self
    }

    pub fn light(mut self, light: Light) -> Self {
        self.lights.push(light);

        self
    }

    pub fn model(mut self, model: Box<dyn Hitable + Sync + Send>) -> Self {
        self.models.push(model);

        self
    }

    pub fn rays(mut self, rays: u32) -> Self {
        self.n_rays = rays;

        self
    }

    pub fn reflects(mut self, reflects: u32) -> Self {
        self.n_reflects = reflects;

        self
    }

    pub fn threads(mut self, threads: u32) -> Self {
        self.n_threads = threads;

        self
    }

    pub fn strategy(mut self, strategy: ChunkStrategy) -> Self {
        self.strategy = strategy;

        self
    }

    pub async fn build(mut self) -> Tracer {
        let image_buffer = ImageBuffer::new(self.width, self.height, self.strategy);

        let material = MaterialBuilder::new("diffuse")
            .diffuse_buffer(&image_buffer.framebuffer, self.width, self.height)
            .await
            .build();

        let image: Arc<Model> = ModelBuilder::new()
            .add_mesh(Shapes::quad(), Some(material.clone()))
            .build(self.shader.clone());

        self.scene.add_node(
            Tracer::LAYER,
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [self.width as f32, self.height as f32, 1.].into(),
            image,
        );

        Tracer {
            scene: self.scene,
            image_buffer,
            models: self.models,
            lights: self.lights,
            camera: self.camera,
            material,
            shader: self.shader,
            background: self.background,
            n_rays: self.n_rays,
            n_reflects: self.n_reflects,
            n_threads: self.n_threads,
            changed: true,
            timer: Timer::new(),
        }
    }
}
