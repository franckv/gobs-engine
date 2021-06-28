use std::sync::Arc;

use simplelog::{Config, LevelFilter, TermLogger};

use gobs_game as game;
use gobs_scene as scene;
use gobs_render as render;

use game::app::{Application, Run};
use scene::Camera;
use scene::model::{Color, ModelBuilder, Shapes, Texture, Transform, Vertex};

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

        let transform = Transform::from_matrix(self.camera.combined());

        engine.renderer().update_view_proj(transform.into());
        engine.renderer().draw_frame(instances);
        engine.renderer().submit_frame();
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.camera.resize(2. * scale, 2.);
    }
}

impl App {
    pub fn new(_engine: &Application) -> Self {
        let mut camera = Camera::new([0., 0., 0.]);
        camera.set_ortho(-10., 10.);
        camera.look_at([0., 0., -1.], [0., 1., 0.]);
        camera.resize(4., 4.);

        App {
            camera,
            triangle: None
        }
    }

    fn draw_triangle(&self) -> Vec<(Arc<ModelCache<Vertex>>, Transform)> {
        vec![(self.triangle.as_ref().unwrap().clone(), Transform::new())]
    }
}

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).expect("error");

    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
