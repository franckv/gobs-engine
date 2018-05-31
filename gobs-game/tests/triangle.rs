extern crate gobs_game as game;
extern crate gobs_scene as scene;

use game::app::{Application, Run};
use game::asset::AssetManager;
use scene::SceneGraph;
use scene::model::{Color, RenderObjectBuilder};

struct App {
    graph: SceneGraph
}

impl Run for App {
    fn create(&mut self, _engine: &mut Application) {
        let texture = AssetManager::get_color_texture(Color::red());
        let triangle = AssetManager::build_triangle();

        let instance = RenderObjectBuilder::new(triangle).texture(texture).build();

        self.graph.insert(instance);
    }

    fn update(&mut self, engine: &mut Application) {
        let batch = engine.batch_mut();

        batch.begin();
        batch.draw_graph(&mut self.graph);
        batch.end();
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.graph.camera_mut().resize(2. * scale, 2.);
    }
}

impl App {
    pub fn new() -> Self {
        App {
            graph: SceneGraph::new()
        }
    }
}

#[test]
pub fn triangle() {
    let mut engine = Application::new();
    engine.run(App::new());
}
