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
use gobs_render::{context::Gfx, pipeline::PipelineFlag, shader::Shader};
use gobs_scene::{
    shape::Shapes, Material, MaterialBuilder, Model, ModelBuilder, RenderError, Scene,
};
use gobs_utils::{rng::RngPool, timer::Timer};

pub struct Tracer {
    scene: Scene,
    image_buffer: ImageBuffer,
    models: Vec<Box<dyn Hitable + Sync + Send>>,
    camera: Camera,
    material: Arc<Material>,
    shader: Arc<Shader>,
    background: fn(&Ray) -> Color,
    n_rays: u32,
    changed: bool,
    timer: Timer,
}

impl Tracer {
    const LAYER: &'static str = "tracer";
    const SHADER: &'static str = "ui.wgsl";
    const MAX_REFLECT: u32 = 10;
    const MIN_DISTANCE: f32 = 0.1;
    const MAX_DISTANCE: f32 = 200.;
    const MULTI_THREAD: bool = true;
    const N_THREADS: u32 = 8;

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

        let image_buffer = ImageBuffer::new(width, height, ChunkStrategy::BOX);

        let material = MaterialBuilder::new("diffuse")
            .diffuse_buffer(&image_buffer.framebuffer, width as u32, height as u32)
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
            scene,
            image_buffer,
            models: Vec::new(),
            camera,
            material,
            shader,
            background,
            n_rays,
            changed: true,
            timer: Timer::new(),
        }
    }

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

    pub fn add_model(&mut self, model: Box<dyn Hitable + Sync + Send>) {
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
        let chunks: Vec<Vec<usize>> = (0..Self::N_THREADS)
            .filter_map(|_| match self.image_buffer.is_complete() {
                true => None,
                false => Some(self.image_buffer.get_chunk()),
            })
            .collect();

        let results: Vec<Vec<(usize, Color)>> = if Self::MULTI_THREAD {
            chunks
                .par_iter()
                .cloned()
                .map(|chunk| self.compute_chunk(chunk))
                .collect()
        } else {
            chunks
                .iter()
                .cloned()
                .map(|chunk| self.compute_chunk(chunk))
                .collect()
        };

        for result in results {
            for (idx, c) in result {
                self.image_buffer.update_pixel(idx, c);
            }
        }
    }

    pub fn compute_chunk(&self, chunk: Vec<usize>) -> Vec<(usize, Color)> {
        let mut result = Vec::new();

        let mut rng = RngPool::new(chunk.len());

        for idx in chunk {
            let c = self.compute_pixel(idx, &mut rng);

            result.push((idx, c));
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
