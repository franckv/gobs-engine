extern crate examples;
extern crate gobs_game as game;
extern crate gobs_render as render;
extern crate gobs_scene as scene;
extern crate gobs_utils as utils;

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::{Config, LevelFilter, TermLogger};

use game::app::{Application, Run};
use utils::timer::Timer;
use render::{Batch, Renderer};
use scene::SceneGraph;
use scene::model::{Font, Transform};

struct App {
    graph: SceneGraph,
    renderer: Renderer,
    batch: Batch,
    timer: Timer
}

impl Run for App {
    fn create(&mut self, _engine: &mut Application) {
        self.draw();
    }

    fn update(&mut self, engine: &mut Application) {
        debug!("Update: {} ms", self.timer.delta() / 1_000_000);
        let cmd = self.batch.draw_graph(&mut self.graph);
        debug!("Batch: {} ms", self.timer.delta() / 1_000_000);
        self.renderer.submit(cmd);
        debug!("Rendering: {} ms", self.timer.delta() / 1_000_000);
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
            timer: Timer::new()
        }
    }

    pub fn draw(&mut self) {
        let font = Font::new(40, &examples::asset("font.ttf"));

        let offset = -1.0;
        for i in 1..40 {
            let i = i as f32;

            let mut group = SceneGraph::new_node()
                .transform(Transform::translation([-1.75, offset + i * 0.05, 0.]))
                .build();

            let chars = font.layout("The quick brown fox jumps over the lazy dog. \
            The quick brown fox jumps over the lazy dog. \
            The quick brown fox jumps over the lazy dog. \
            The quick brown fox jumps over the lazy dog.");

            for (c, trans) in chars {
                let c = SceneGraph::new_node()
                    .data(c)
                    .model_transform(trans)
                    .build();
                group.insert(c);
            }

            self.graph.insert(group);
        }
    }
}

pub fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).expect("error");
    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
