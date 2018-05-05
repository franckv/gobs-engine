use render::Batch;
use render::display;
use render::display::Display;

use asset::AssetManager;
use input::{Event, InputHandler, InputMap};

pub struct Application {
    batch: Batch,
    display: Display,
    asset: AssetManager,
    input_handler: InputHandler
}

impl Application {
    pub fn new() -> Application {
        let (events_loop, display, renderer) = display::init();

        let asset = AssetManager::new(&renderer);
        let batch = Batch::new(renderer);
        let input_handler = InputHandler::new(events_loop);

        Application {
            batch: batch,
            display: display,
            asset: asset,
            input_handler: input_handler
        }
    }

    pub fn batch_mut(&mut self) -> &mut Batch {
        &mut self.batch
    }

    pub fn input_map(&self) -> &InputMap {
        &self.input_handler.get_input_map()
    }

    pub fn asset_manager_mut(&mut self) -> &mut AssetManager {
        &mut self.asset
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.display.get_dimensions()
    }

    pub fn run<R>(&mut self, mut runnable: R) where R: Run {
        runnable.create(self);

        let mut running = true;

        while running {
            let event = self.input_handler.read_inputs();

            match event {
                Event::RESIZE => {
                    let (width, height) = self.display.get_dimensions();

                    self.batch.renderer_mut().resize(width, height);
                    runnable.resize(width, height, self);
                },
                Event::CLOSE => {
                    running = false;
                },
                Event::CONTINUE => ()
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
