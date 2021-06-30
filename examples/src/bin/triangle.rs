use std::sync::Arc;

use simplelog::{Config, LevelFilter, TermLogger};

use gobs_game as game;
use gobs_scene as scene;
use gobs_render as render;

use game::app::{Application, Run};
use scene::Camera;
use scene::model::{Color, ModelBuilder, Shapes, Texture, Transform};

use render::instance::ModelInstance;

struct App {
    camera: Camera,
    triangle: Option<Arc<ModelInstance>>,
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        let texture = Texture::from_color(Color::red());
        let triangle = Shapes::triangle();
        let model = ModelBuilder::new(triangle).texture(texture).build();

        self.triangle = Some(ModelInstance::new(&engine.renderer().context, &model));
    }

    fn update(&mut self, _delta: u64, engine: &mut Application) {
        if !engine.renderer().new_frame().is_ok() {
            return;
        }

        let instances = self.draw_triangle();

        engine.renderer().draw_frame(instances, &self.camera);
        engine.renderer().submit_frame();
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.camera.set_aspect(scale);
    }
}

impl App {
    pub fn new(engine: &Application) -> Self {
        let dim = engine.dimensions();
        let scale = dim.0 as f32 / dim.1 as f32;

        let mut camera = Camera::ortho_fixed_height(2., scale);

        camera.look_at([0., 0., -1.], [0., 1., 0.]);

        App {
            camera,
            triangle: None
        }
    }

    fn draw_triangle(&self) -> Vec<(Arc<ModelInstance>, Transform)> {
        vec![(self.triangle.as_ref().unwrap().clone(), Transform::new())]
    }
}

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).expect("error");

    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
