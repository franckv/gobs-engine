use std::sync::Arc;

use utils::timer::Timer;
use winit::{EventsLoop, WindowBuilder};

use api::renderer::Renderer;
use api::context::Context;

use api::handler::{Event, InputHandler};
use api::input::InputMap;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MAX_INSTANCES: usize = 81;
const MAX_DRAWS: usize = 64;

pub struct Application {
    context: Arc<Context>,
    renderer: Renderer,
    input_handler: InputHandler
}

impl Application {
    pub fn new() -> Application {
        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_dimensions((WIDTH, HEIGHT).into())
            .with_title("Test")
            .with_resizable(false)
            .build(&events_loop).unwrap();

        let input_handler = InputHandler::new(events_loop);

        debug!("Create Context");
        let (context, display) = Context::new("Test", window);

        debug!("Create Renderer");
        let renderer = Renderer::new(context.clone(), display,
        MAX_INSTANCES, MAX_DRAWS);

        Application {
            context,
            renderer,
            input_handler
        }
    }

    pub fn renderer(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn input_map(&self) -> &InputMap {
        &self.input_handler.get_input_map()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.renderer.display.dimensions()
    }

    pub fn run<R>(&mut self, mut runnable: R) where R: Run {
        runnable.create(self);

        let mut running = true;
        let mut timer = Timer::new();

        while running {
            let delta = timer.delta();
            debug!("FPS: {}", 1_000_000_000 / delta);

            let event = self.input_handler.read_inputs();

            match event {
                Event::Resize => {
                    let (width, height) = self.renderer.display.dimensions();

                    runnable.resize(width, height, self);
                },
                Event::Close => {
                    running = false;
                },
                Event::Continue => ()
            }

            runnable.update(delta, self);
        }

        self.renderer.context.device_ref().wait();
    }
}

pub trait Run: Sized {
    fn create(&mut self, _application: &mut Application) {}
    fn update(&mut self, _delta: u64, _application: &mut Application) {}
    fn resize(&mut self, _width: u32, _height: u32, _application: &mut Application) {}
}