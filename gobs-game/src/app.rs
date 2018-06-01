use std::sync::Arc;

use render::Batch;
use render::context;
use render::context::Context;
use render::display::Display;

use input::{Event, InputHandler, InputMap};

pub struct Application {
    context: Arc<Context>,
    display: Arc<Display>,
    input_handler: InputHandler
}

impl Application {
    pub fn new() -> Application {
        let (events_loop, context, display) = context::init();

        let input_handler = InputHandler::new(events_loop);

        Application {
            context: context,
            display: display,
            input_handler: input_handler
        }
    }

    pub fn create_batch(&self) -> Batch {
        Batch::new(self.display.clone(), self.context.clone())
    }

    pub fn input_map(&self) -> &InputMap {
        &self.input_handler.get_input_map()
    }

    pub fn dimensions(&self) -> [u32; 2] {
        self.display.dimensions()
    }

    pub fn run<R>(&mut self, mut runnable: R) where R: Run {
        runnable.create(self);

        let mut running = true;

        while running {
            let event = self.input_handler.read_inputs();

            match event {
                Event::Resize => {
                    let [width, height] = self.display.dimensions();

                    runnable.resize(width, height, self);
                },
                Event::Close => {
                    running = false;
                },
                Event::Continue => ()
            }

            runnable.update(self);
        }
    }
}

pub trait Run: Sized {
    fn create(&mut self, _application: &mut Application) {}
    fn update(&mut self, _application: &mut Application) {}
    fn resize(&mut self, _width: u32, _height: u32, _application: &mut Application) {}
}
