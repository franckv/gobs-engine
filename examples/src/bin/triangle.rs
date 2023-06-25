use std::sync::Arc;

use simplelog::{Config, LevelFilter, TermLogger, ColorChoice, TerminalMode};

use gobs_game as game;
use gobs_scene as scene;
use gobs_render as render;

use game::app::{Application, Run};
use scene::model::{Color, ModelBuilder, Shapes, Texture};
use scene::SceneGraph;
use render::instance::ModelInstance;

struct App {
    graph: SceneGraph<Arc<ModelInstance>>
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        let texture = Texture::from_color(Color::red());
        let triangle = Shapes::triangle();
        let model = ModelBuilder::new(triangle).texture(texture).build();

        let triangle = ModelInstance::new(&engine.renderer().context, &model);

        self.graph.insert(SceneGraph::new_node().data(triangle).build());
    }

    fn update(&mut self, _delta: f32, engine: &mut Application) {
        if !engine.renderer().new_frame().is_ok() {
            return;
        }

        engine.renderer().draw_frame(&mut self.graph);
        engine.renderer().submit_frame();
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.graph.camera_mut().set_aspect(scale);
    }
}

impl App {
    pub fn new(_engine: &Application) -> Self {
        let mut graph = SceneGraph::new();

        graph.camera_mut().set_mode(scene::scene::camera::ProjectionMode::OrthoFixedHeight);
        graph.camera_mut().resize(2., 2.);
        graph.camera_mut().look_at([0., 0., -1.], [0., 1., 0.]);

        App {
            graph
        }
    }
}

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).expect("error");

    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
