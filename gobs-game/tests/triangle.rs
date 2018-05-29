extern crate gobs_render as render;
extern crate gobs_game as game;

use std::sync::Arc;

use game::app::{Application, Run};
use render::model::{Color, MeshInstanceBuilder};
use render::scene::SceneGraph;

struct App {
    graph: SceneGraph
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        let asset_manager = engine.asset_manager_mut();

        let texture = asset_manager.get_color_texture(Color::red());
        let triangle = asset_manager.build_triangle();

        let instance = MeshInstanceBuilder::new(triangle).texture(texture).build();

        self.graph.add_instance(Arc::new(instance));
    }

    fn update(&mut self, engine: &mut Application) {
        let batch = engine.batch_mut();

        batch.begin();
        batch.draw_graph(&self.graph);
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
