use std::sync::Arc;

use simplelog::{Config, LevelFilter, TermLogger, ColorChoice, TerminalMode};

use gobs_game as game;
use gobs_scene as scene;
use gobs_render as render;
use gobs_utils as utils;

use game::app::{Application, Run};
use scene::model::{Color, Mesh, ModelBuilder,
                   Shapes, Texture, Transform};
use scene::SceneGraph;
use render::context::Context;
use render::instance::ModelInstance;

use utils::timer::Timer;

const N_TRIANGLES: usize = 9;

#[allow(dead_code)]
struct App {
    graph: SceneGraph<Arc<ModelInstance>>,
    frame: usize
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        Self::draw_triangles(&mut self.graph, &engine.renderer().context, N_TRIANGLES);
    }

    fn update(&mut self, _delta: f32, engine: &mut Application) {
        if !engine.renderer().new_frame().is_ok() {
            return;
        }

        let model_transform = Transform::rotation([1., 0., 0.], 1.);

        self.graph.foreach(|data, _transform| {
            data.transform_mut().transform(&model_transform);
        });

        self.graph.set_dirty();

        let view_transform = Transform::rotation([0., 1., 0.], 0.5);

        self.graph.camera_mut().transform(view_transform.into());

        engine.renderer().draw_frame(&mut self.graph);
        engine.renderer().submit_frame();

        self.frame += 1;
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        let scale = width as f32 / height as f32;
        self.graph.camera_mut().set_aspect(scale);
    }
}

#[allow(dead_code)]
impl App {
    pub fn new(_engine: &Application) -> Self {
        let mut graph = SceneGraph::new();

        graph.camera_mut().set_mode(scene::scene::camera::ProjectionMode::OrthoFixedHeight);
        graph.camera_mut().resize(4., 4.);
        graph.camera_mut().look_at([0., 0., -1.], [0., 1., 0.]);

        App {
            graph,
            frame: 0
        }
    }

    fn build_model(context: &Arc<Context>, shape: Arc<Mesh>,
                   texture: Arc<Texture>) -> Arc<ModelInstance> {
        let object = ModelBuilder::new(shape)
            .texture(texture).build();

            ModelInstance::new(context, &object)
    }

    fn draw_triangles(graph: &mut SceneGraph<Arc<ModelInstance>>, context: &Arc<Context>, rows: usize) {
        let red = Texture::from_color(Color::red());
        let blue = Texture::from_color(Color::blue());

        let triangle = Shapes::triangle();

        let triangle_r = Self::build_model(context, triangle.clone(), red.clone());
        let triangle_b = Self::build_model(context, triangle.clone(), blue.clone());

        let mut timer = Timer::new();

        let width = 3.5;
        let step = width / (rows-1) as f32;

        let mut positions = Vec::new();

        log::debug!("Triangles: {}", timer.delta());

        for i in 0..rows {
            for j in 0..rows {
                positions.push([
                    -width / 2. + i as f32 * step,
                    -width / 2. + j as f32 * step,
                    0.,
                ]);
            }
        }

        log::debug!("Positions: {}", timer.delta());

        let scale = width / (2 * rows) as f32;

        let mut even = false;

        for position in positions {
            let instance = match even {
                true => triangle_r.clone(),
                false => triangle_b.clone(),
            };
            even = !even;

            let transform = Transform::scaling(scale, scale, 1.)
                .translate(position);

            graph.insert(SceneGraph::new_node().data(instance).transform(transform).build());

        }

        log::debug!("Instances: {}", timer.delta());
    }
}

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).expect("error");

    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}
