use log::*;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use gobs_scene as scene;
use gobs_utils as utils;

use utils::timer::Timer;

use crate::input::{Event, Input};

use scene::Gfx;
use scene::RenderError;

const WIDTH: u32 = 1920; // TODO: hardcoded
const HEIGHT: u32 = 1080;

pub struct Application {
    pub window: Window,
    pub events_loop: EventLoop<()>,
    pub gfx: Gfx,
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

        log::debug!("Create Gfx");
        let gfx = pollster::block_on(Gfx::new(&window));

        Application {
            window,
            events_loop,
            gfx,
        }
    }

    pub fn renderer(&mut self) -> &mut Gfx {
        &mut self.gfx
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.gfx.width(), self.gfx.height())
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
        let mut runnable = R::create(&self.gfx).await;

        self.events_loop.run(move |event, _, control_flow| {
            log::trace!("evt={:?}, ctrl={:?}", event, control_flow);

            let event = Event::new(event);
            match event {
                Event::Resize(width, height) => {
                    log::debug!("Resize to : {}/{}", width, height);
                    self.gfx.resize(width, height);

                    runnable.resize(width, height, &self.gfx);
                }
                Event::Input(input) => {
                    runnable.input(&self.gfx, input);
                }
                Event::Close => {
                    log::info!("Stopping");
                    *control_flow = ControlFlow::Exit;
                }
                Event::Redraw => {
                    let delta = timer.delta();
                    log::trace!("FPS: {}", 1. / delta);

                    runnable.update(delta, &self.gfx);
                    match runnable.render(&self.gfx) {
                        Ok(_) => {}
                        Err(RenderError::Lost | RenderError::Outdated) => {
                            self.gfx.resize(self.gfx.width(), self.gfx.height());
                            runnable.resize(self.gfx.width(), self.gfx.height(), &self.gfx);
                        }
                        Err(e) => error!("{:?}", e),
                    }
                }
                Event::Cleared => {
                    self.window.request_redraw();
                }
                Event::Continue => (),
            }
        });
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(async_fn_in_trait)]
pub trait Run: Sized {
    async fn create(gfx: &Gfx) -> Self;
    fn update(&mut self, delta: f32, gfx: &Gfx);
    fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError>;
    fn input(&mut self, gfx: &Gfx, input: Input);
    fn resize(&mut self, width: u32, height: u32, gfx: &Gfx);
}
