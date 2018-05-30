extern crate gobs_game as game;
extern crate gobs_scene as scene;

use std::sync::Arc;

use game::app::{Application, Run};
use game::asset::AssetManager;
use scene::SceneGraph;
use scene::model::MeshInstanceBuilder;

struct App {
    graph: SceneGraph
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        self.draw();
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

    pub fn draw(&mut self) {
        let font = AssetManager::load_font(42, "../../assets/font.ttf");

        let square = AssetManager::build_quad();

        let chars = font.layout("The quick brown fox jumps over the lazy dog");

        for c in chars {
            let mut instance = MeshInstanceBuilder::new(square.clone())
                .texture(font.texture())
                .region(*c.region())
                .build();

            instance.transform(c.transform());
            instance.translate((-0.5, 0., 0.));
            //instance.scale(2.0, 2.0, 1.0);
            self.graph.add_instance(Arc::new(instance));
        }
    }
}

#[test]
pub fn font() {
    let mut engine = Application::new();
    engine.run(App::new());
}
