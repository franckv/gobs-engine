use log::*;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use gobs_utils as utils;
use gobs_wgpu as render;

use utils::timer::Timer;

use crate::input::{Event, Input};

use render::render::Gfx;

const WIDTH: u32 = 800; // TODO: hardcoded
const HEIGHT: u32 = 600;

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

    async fn run_async<R>(mut self)
    where
        R: Run + 'static,
    {
        let mut timer = Timer::new();
        let mut runnable = R::create(&mut self.gfx).await;

        self.events_loop.run(move |event, _, control_flow| {
            let event = Event::new(event);
            match event {
                Event::Resize(width, height) => {
                    log::debug!("Resize to : {}/{}", width, height);
                    self.gfx.resize(width, height);

                    runnable.resize(width, height, &mut self.gfx);
                }
                Event::Input(input) => {
                    runnable.input(&mut self.gfx, input);
                }
                Event::Close => {
                    log::info!("Stopping");
                    *control_flow = ControlFlow::Exit
                }
                Event::Redraw => {
                    let delta = timer.delta();
                    log::trace!("FPS: {}", 1.0 / delta);

                    runnable.update(delta, &mut self.gfx);
                    match runnable.render(&mut self.gfx) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            self.gfx.resize(self.gfx.width(), self.gfx.height());
                            runnable.resize(self.gfx.width(), self.gfx.height(), &mut self.gfx);
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

pub trait Run: Sized {
    #[allow(async_fn_in_trait)]
    async fn create(gfx: &mut Gfx) -> Self;
    fn update(&mut self, delta: f32, gfx: &mut Gfx);
    fn render(&mut self, gfx: &mut Gfx) -> Result<(), wgpu::SurfaceError>;
    fn input(&mut self, gfx: &mut Gfx, input: Input);
    fn resize(&mut self, width: u32, height: u32, gfx: &mut Gfx);
}
