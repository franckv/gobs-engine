extern crate gobs_game as game;
extern crate gobs_render as render;
extern crate gobs_scene as scene;
extern crate cgmath;

use cgmath::Point3;

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
        self.draw_centers();
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

    fn draw_centers(&mut self) {
        let texture = Texture::from_color(Color::green());

        let left: Point3<f32> = [-1., 0., 0.5].into();
        let right: Point3<f32> = [1., 0., 0.5].into();
        let top: Point3<f32> = [0., 1., 0.5].into();
        let bottom: Point3<f32> = [0., -1., 0.5].into();

        let line = Shapes::line(left, right);
        let instance = RenderObjectBuilder::new(line).texture(texture.clone()).build();
        self.graph.insert(instance);

        let line = Shapes::line(bottom, top);
        let instance = RenderObjectBuilder::new(line).texture(texture).build();
        self.graph.insert(instance);
    }
}

pub fn main() {
    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
