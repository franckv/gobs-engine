extern crate cgmath;

extern crate gobs_game as game;
extern crate gobs_scene as scene;

use std::sync::Arc;

use cgmath::Point3;

use game::app::{Application, Run};
use game::asset::AssetManager;
use scene::SceneGraph;
use scene::model::{Color, MeshInstanceBuilder};

struct App {
    graph: SceneGraph
}

impl Run for App {
    fn create(&mut self, _engine: &mut Application) {
        self.draw_centers();
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

    fn draw_centers(&mut self) {
        let texture = AssetManager::get_color_texture(Color::green());

        let left: Point3<f32> = [-1., 0., 0.5].into();
        let right: Point3<f32> = [1., 0., 0.5].into();
        let top: Point3<f32> = [0., 1., 0.5].into();
        let bottom: Point3<f32> = [0., -1., 0.5].into();

        let line = AssetManager::build_line(left, right);
        let instance = MeshInstanceBuilder::new(line).texture(texture.clone()).build();
        self.graph.add_instance(Arc::new(instance));

        let line = AssetManager::build_line(bottom, top);
        let instance = MeshInstanceBuilder::new(line).texture(texture).build();
        self.graph.add_instance(Arc::new(instance));
    }
}

#[test]
pub fn line() {
    let mut engine = Application::new();
    engine.run(App::new());
}
