extern crate gobs_game as game;
extern crate gobs_render as render;
extern crate gobs_scene as scene;
extern crate cgmath;

use game::app::{Application, Run};
use render::{Batch, Renderer};
use scene::SceneGraph;
use scene::model::{Color, RenderObjectBuilder, Shapes, Texture};

struct App {
    graph: SceneGraph,
    renderer: Renderer,
    batch: Batch
}

impl Run for App {
    fn create(&mut self, _engine: &mut Application) {
        let texture = Texture::from_color(Color::red());
        let triangle = Shapes::triangle();

        let instance = RenderObjectBuilder::new(triangle).texture(texture).build();

        self.graph.insert(instance);
    }

    fn update(&mut self, engine: &mut Application) {
        let cmd = self.batch.draw_graph(&mut self.graph);
        self.renderer.submit(cmd);
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.graph.camera_mut().resize(2. * scale, 2.);
    }
}

impl App {
    pub fn new(engine: &Application) -> Self {
        let renderer = engine.create_renderer();

        App {
            graph: SceneGraph::new(),
            batch: renderer.create_batch(),
            renderer: renderer
        }
    }
}

pub fn main() {
    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}