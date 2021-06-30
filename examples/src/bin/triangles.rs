use std::sync::Arc;

use simplelog::{Config, LevelFilter, TermLogger};

use gobs_game as game;
use gobs_scene as scene;
use gobs_render as render;
use gobs_utils as utils;

use game::app::{Application, Run};
use scene::Camera;
use scene::model::{Color, Mesh, ModelBuilder,
                   Shapes, Texture, Transform};
use render::context::Context;
use render::instance::ModelInstance;

use utils::timer::Timer;

const N_TRIANGLES: usize = 9;

#[allow(dead_code)]
struct App {
    camera: Camera,
    triangle_r: Option<Arc<ModelInstance>>,
    triangle_b: Option<Arc<ModelInstance>>,
    square: Option<Arc<ModelInstance>>,
    frame: usize
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        let red = Texture::from_color(Color::red());
        let blue = Texture::from_color(Color::blue());

        let triangle = Shapes::triangle();
        let square = Shapes::quad();

        let triangle_r = Self::build_model(&engine.renderer().context,
                                           triangle.clone(), red.clone());
        let triangle_b = Self::build_model(&engine.renderer().context,
                                           triangle.clone(), blue.clone());
        let square = Self::build_model(&engine.renderer().context,
                                       square.clone(), blue.clone());

        self.triangle_r = Some(triangle_r);
        self.triangle_b = Some(triangle_b);
        self.square = Some(square);
    }

    fn update(&mut self, _delta: u64, engine: &mut Application) {
        if !engine.renderer().new_frame().is_ok() {
            return;
        }

        let instances =
            self.draw_triangles(N_TRIANGLES);

        let view_transform = Transform::rotation([0., 1., 0.], 0.5);

        self.camera.transform(view_transform.into());

        engine.renderer().draw_frame(instances, &self.camera);
        engine.renderer().submit_frame();

        self.frame += 1;
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.camera.set_aspect(scale);
    }
}

#[allow(dead_code)]
impl App {
    pub fn new(engine: &Application) -> Self {
        let dim = engine.dimensions();
        let scale = dim.0 as f32 / dim.1 as f32;

        let mut camera = Camera::ortho_fixed_height(4., scale);

        camera.look_at([0., 0., -1.], [0., 1., 0.]);

        App {
            camera,
            triangle_r: None,
            triangle_b: None,
            square: None,
            frame: 0
        }
    }

    fn build_model(context: &Arc<Context>, shape: Arc<Mesh>,
                   texture: Arc<Texture>) -> Arc<ModelInstance> {
        let object = ModelBuilder::new(shape)
            .texture(texture).build();

            ModelInstance::new(context, &object)
    }

    fn draw_triangle(&self) -> Vec<(Arc<ModelInstance>, Transform)> {
        vec![(self.triangle_r.as_ref().unwrap().clone(), Transform::new())]
    }

    fn draw_triangles(&self, rows: usize) -> Vec<(Arc<ModelInstance>, Transform)> {
        let mut timer = Timer::new();

        let offset = self.frame as f32;

        let width = 3.5;
        let step = width / (rows-1) as f32;

        let mut positions = Vec::new();

        log::debug!("Triangles: {}", timer.delta() / 1_000_000);

        for i in 0..rows {
            for j in 0..rows {
                positions.push([
                    -width / 2. + i as f32 * step,
                    -width / 2. + j as f32 * step,
                    0.,
                ]);
            }
        }

        log::debug!("Positions: {}", timer.delta() / 1_000_000);

        let scale = width / (2 * rows) as f32;

        let mut even = false;

        let instances = positions.iter().map(|position| {
            let instance = match even {
                true => self.triangle_r.as_ref().unwrap().clone(),
                false => self.triangle_b.as_ref().unwrap().clone(),
            };
            even = !even;
            let transform = Transform::scaling(scale, scale, 1.)
                .transform(&Transform::rotation([1., 0., 0.], offset))
                .translate(*position);

            let instance = (instance, transform);

            instance
        }).collect();

        log::debug!("Instances: {}", timer.delta() / 1_000_000);

        instances
    }
}

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).expect("error");

    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
