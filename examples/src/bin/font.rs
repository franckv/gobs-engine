extern crate examples;
extern crate gobs_game as game;
extern crate gobs_render as render;
extern crate gobs_scene as scene;
extern crate cgmath;

use cgmath::Matrix4;

use game::app::{Application, Run};
use render::{Batch, Renderer};
use scene::SceneGraph;
use scene::model::Font;

struct App {
    graph: SceneGraph,
    renderer: Renderer,
    batch: Batch
}

impl Run for App {
    fn create(&mut self, _engine: &mut Application) {
        self.draw();
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
            renderer: renderer,
        }
    }

    pub fn draw(&mut self) {
        let font = Font::new(42, &examples::asset("font.ttf"));

        let chars = font.layout("The quick brown fox jumps over the lazy dog");

        for mut c in chars {
            let transform = Matrix4::from_translation([-0.5, 0., 0.].into());
            self.graph.insert_with_transform(c, transform);

        }
    }
}

pub fn main() {
    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}