use log::*;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use gobs_utils as utils;

use utils::timer::Timer;

use crate::context::Context;
use crate::input::{Event, Input};

const WIDTH: u32 = 800; // TODO: hardcoded
const HEIGHT: u32 = 600;

pub struct Application {
    pub context: Context,
    pub events_loop: EventLoop<()>,
}

impl Application {
    pub fn new() -> Application {
        let events_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_title("Test")
            .with_resizable(true)
            .build(&events_loop)
            .unwrap();

        let context = Context::new("engine", window);

        Application {
            context,
            events_loop,
        }
    }

    pub fn run<R>(self)
    where
        R: Run + 'static,
    {
        pollster::block_on(self.run_async::<R>());
    }

    async fn run_async<R>(self)
    where
        R: Run + 'static,
    {
        let mut timer = Timer::new();
        let mut runnable = R::create(&self.context).await;

        log::info!("Start main loop");

        self.events_loop.run(move |event, _, control_flow| {
            log::trace!("evt={:?}, ctrl={:?}", event, control_flow);

            let event = Event::new(event);
            match event {
                Event::Resize(width, height) => {
                    log::debug!("Resize to : {}/{}", width, height);
                    runnable.resize(&self.context, width, height);
                }
                Event::Input(input) => {
                    runnable.input(&self.context, input);
                }
                Event::Close => {
                    log::info!("Stopping");
                    runnable.close(&self.context);
                    *control_flow = ControlFlow::Exit;
                }
                Event::Redraw => {
                    let delta = timer.delta();
                    log::debug!("[Redraw] FPS: {}", 1. / delta);

                    runnable.update(&self.context, delta);
                    match runnable.render(&self.context) {
                        Ok(_) => {}
                        Err(RenderError::Lost | RenderError::Outdated) => {}
                        Err(e) => error!("{:?}", e),
                    }
                }
                Event::Cleared => {
                    self.context.surface.window.request_redraw();
                }
                Event::Continue => (),
            }
        });
    }
}

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Error,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(async_fn_in_trait)]
pub trait Run: Sized {
    async fn create(context: &Context) -> Self;
    fn update(&mut self, context: &Context, delta: f32);
    fn render(&mut self, context: &Context) -> Result<(), RenderError>;
    fn input(&mut self, context: &Context, input: Input);
    fn resize(&mut self, context: &Context, width: u32, height: u32);
    fn close(&mut self, context: &Context);
}
