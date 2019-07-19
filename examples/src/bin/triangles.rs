#[macro_use]
extern crate log;
extern crate simplelog;
extern crate time;
extern crate winit;
extern crate gobs_vulkan;
extern crate gobs_scene;
extern crate gobs_utils;

use std::sync::Arc;

use simplelog::{Config, LevelFilter, TermLogger};

use gobs_vulkan as vulkan;
use gobs_scene as scene;
use gobs_utils as utils;

use vulkan::api as api;

use api::app::{Application, Run};
use api::context::Context;
use api::model::Model;
use api::model_instance::ModelInstance;

use scene::{Camera, SceneGraph};
use scene::model::{Color, Mesh, RenderObjectBuilder,
                   Shapes, Texture, Transform, Vertex};

use utils::timer::Timer;

const N_TRIANGLES: usize = 9;

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default()).unwrap();

    debug!("Starting");
    let mut engine = Application::new();
    let app = App::new(&engine);
    engine.run(app);
}

#[allow(dead_code)]
struct App {
    camera: Camera,
    triangle_r: Option<Arc<Model<Vertex>>>,
    triangle_b: Option<Arc<Model<Vertex>>>,
    square: Option<Arc<Model<Vertex>>>,
    frame: usize,
    graph: SceneGraph
}

impl Run for App {
    fn create(&mut self, engine: &mut Application) {
        debug!("Creating App");

        let red = Texture::from_color(Color::red());
        let blue = Texture::from_color(Color::blue());

        let triangle = Shapes::triangle();
        let square = Shapes::quad();

        let triangle_r = Self::build_model(&engine.renderer().context,
                                           triangle.clone(), red.clone());
        let triangle_b = Self::build_model(&engine.renderer().context,
                                           triangle.clone(), blue.clone());
        let square = Self::build_model(&engine.renderer().context,
                                       square.clone(), blue.clone());

        self.triangle_r = Some(triangle_r);
        self.triangle_b = Some(triangle_b);
        self.square = Some(square);

        debug!("App created");
    }

    fn update(&mut self, _delta: u64, engine: &mut Application) {
        let mut timer = Timer::new();

        if !engine.renderer().new_frame().is_ok() {
            return;
        }

        debug!("Wait: {}", timer.delta() / 1_000_000);

        let instances =
            self.draw_triangles(N_TRIANGLES);

        debug!("Build scene: {}", timer.delta() / 1_000_000);

        let offset = self.frame as f32 / 10.;

        let view_transform = Transform::rotation([0., 1., 0.], offset);
        let proj_transform = Transform::from_matrix(self.camera.combined());

        let view_proj_transform =
            view_transform.transform(&proj_transform);

        engine.renderer().update_view_proj(view_proj_transform.into());

        debug!("Update view: {}", timer.delta() / 1_000_000);

        engine.renderer().draw_frame(instances);

        debug!("Draw scene: {}", timer.delta() / 1_000_000);

        engine.renderer().submit_frame();

        debug!("Submit scene: {}", timer.delta() / 1_000_000);

        self.frame += 1;
    }

    fn resize(&mut self, width: u32, height: u32, _engine: &mut Application) {
        info!("The window was resized to {}x{}", width, height);
        let scale = width as f32 / height as f32;
        self.graph.camera_mut().resize(2. * scale, 2.);
    }
}

#[allow(dead_code)]
impl App {
    pub fn new(_engine: &Application) -> Self {
        debug!("New App");
        let mut camera = Camera::new([0., 0., 0.]);
        camera.set_ortho(-10., 10.);
        camera.look_at([0., 0., -1.], [0., 1., 0.]);
        camera.resize(4. , 4.);

        App {
            camera,
            triangle_r: None,
            triangle_b: None,
            square: None,
            frame: 0,
            graph: SceneGraph::new()
        }
    }

    fn build_model(context: &Arc<Context>, shape: Arc<Mesh>,
                   texture: Arc<Texture>) -> Arc<Model<Vertex>> {
        let object = RenderObjectBuilder::new(shape)
            .texture(texture).build();

        Model::<Vertex>::new(context, &object)
    }

    fn draw_triangle(&self) -> Vec<ModelInstance<Vertex, Transform>> {
        vec![ModelInstance::new(self.triangle_r.as_ref().unwrap().clone(), Transform::new())]
    }

    fn draw_triangles(&self, rows: usize) -> Vec<ModelInstance<Vertex, Transform>> {
        let mut timer = Timer::new();

        let offset = self.frame as f32;

        let width = 3.5;
        let step = width / (rows-1) as f32;

        let mut positions = Vec::new();

        debug!("Triangles: {}", timer.delta() / 1_000_000);

        for i in 0..rows {
            for j in 0..rows {
                positions.push([
                    -width / 2. + i as f32 * step,
                    -width / 2. + j as f32 * step,
                    0.,
                ]);
            }
        }

        debug!("Positions: {}", timer.delta() / 1_000_000);

        let scale = width / (2 * rows) as f32;

        let mut even = false;

        let instances = positions.iter().map(|position| {
            let model = match even {
                true => self.triangle_r.as_ref().unwrap().clone(),
                false => self.triangle_b.as_ref().unwrap().clone(),
            };
            even = !even;
            let transform = Transform::scaling(scale, scale, 1.)
                .transform(&Transform::rotation([1., 0., 0.], offset))
                .translate(*position);

            let instance = ModelInstance::new(model, transform);

            instance
        }).collect();

        debug!("Instances: {}", timer.delta() / 1_000_000);

        instances
    }
}
