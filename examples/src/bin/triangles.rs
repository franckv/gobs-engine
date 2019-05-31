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

use winit::{Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::WindowBuilder;

use gobs_vulkan as vulkan;
use gobs_scene as scene;
use gobs_utils as utils;

use vulkan::api as api;

use api::context::Context;
use api::model::Model;
use api::model_instance::ModelInstance;
use api::renderer::Renderer;

use scene::Camera;
use scene::model::{Color, Mesh, RenderObjectBuilder,
                   Shapes, Texture, Transform, Vertex};

use utils::timer::Timer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const N_TRIANGLES: usize = 9;

const MAX_DRAWS: usize = 64;

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_dimensions((WIDTH, HEIGHT).into())
        .with_title("Test")
        .with_resizable(false)
        .build(&events_loop).unwrap();

    let max_instances = N_TRIANGLES * N_TRIANGLES;

    let renderer = Renderer::new("Test", window,
                                 max_instances, MAX_DRAWS);

    let mut app = App::new(renderer);

    app.run(events_loop);
}

#[allow(dead_code)]
struct App {
    camera: Camera,
    triangle_r: Arc<Model<Vertex>>,
    triangle_b: Arc<Model<Vertex>>,
    square: Arc<Model<Vertex>>,
    renderer: Renderer,
    frame: usize
}

#[allow(dead_code)]
impl App {
    pub fn new(renderer: Renderer) -> Self {
        let mut camera = Camera::new([0., 0., 0.]);
        camera.set_ortho(-10., 10.);
        camera.look_at([0., 0., -1.], [0., 1., 0.]);
        camera.resize(4. , 4.);

        let red = Texture::from_color(Color::red());
        let blue = Texture::from_color(Color::blue());

        let triangle = Shapes::triangle();
        let square = Shapes::quad();

        let triangle_r = Self::build_model(&renderer.context,
                                           triangle.clone(), red.clone());
        let triangle_b = Self::build_model(&renderer.context,
                                           triangle.clone(), blue.clone());
        let square = Self::build_model(&renderer.context,
                                       square.clone(), blue.clone());

        App {
            camera,
            triangle_r,
            triangle_b,
            square,
            renderer,
            frame: 0
        }
    }

    fn build_model(context: &Arc<Context>, shape: Arc<Mesh>,
                   texture: Arc<Texture>) -> Arc<Model<Vertex>> {
        let object = RenderObjectBuilder::new(shape)
            .texture(texture).build();

        Model::<Vertex>::new(context, &object)
    }

    fn draw_triangle(&self) -> Vec<ModelInstance<Vertex, Transform>> {
        vec![ModelInstance::new(self.triangle_r.clone(), Transform::new())]
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
                true => self.triangle_r.clone(),
                false => self.triangle_b.clone(),
            };
            even = !even;
            let transform = Transform::scaling(scale, scale, 1.)
                .transform(&Transform::rotation([1., 0., 0.], offset))
                .translate(*position);

            ModelInstance::new(model, transform)
        }).collect();

        debug!("Instances: {}", timer.delta() / 1_000_000);

        instances
    }

    fn update(&mut self, _delta: u64) {
        let mut timer = Timer::new();

        if !self.renderer.new_frame().is_ok() {
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

        self.renderer.update_view_proj(view_proj_transform.into());

        debug!("Update view: {}", timer.delta() / 1_000_000);

        self.renderer.draw_frame(instances);

        debug!("Draw scene: {}", timer.delta() / 1_000_000);

        self.renderer.submit_frame();

        debug!("Submit scene: {}", timer.delta() / 1_000_000);
    }

    fn run(&mut self, mut events_loop: EventsLoop) {
        let mut running = true;

        let mut timer = Timer::new();

        while running {
            let delta = timer.delta();

            debug!("FPS: {}", 1_000_000_000 / delta);

            events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Resized(size) => {
                            let dpi = self.renderer.display.surface_ref().dpi();
                            let size = size.to_physical(dpi);
                            info!("The window was resized to {}x{}",
                                  size.width, size.height);
                        }
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            }, ..
                        } => running = false,
                        _ => ()
                    },
                    _ => ()
                }
            });

            self.update(delta);

            self.frame += 1;
        }


        self.renderer.context.device_ref().wait();
    }
}
