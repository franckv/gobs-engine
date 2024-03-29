use log::*;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use gobs_render::context::Context;
use gobs_render::graph::RenderError;
use gobs_utils::timer::Timer;

use crate::input::{Event, Input};

pub struct Application {
    pub context: Context,
    pub events_loop: EventLoop<()>,
}

impl Application {
    pub fn new(title: &str, width: u32, height: u32) -> Application {
        let events_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title(title)
            .with_resizable(true)
            .build(&events_loop)
            .unwrap();

        let context = Context::new(title, window);

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

    async fn run_async<R>(mut self)
    where
        R: Run + 'static,
    {
        let mut timer = Timer::new();
        let mut runnable = R::create(&self.context).await;

        log::info!("Start main loop");
        runnable.start(&self.context).await;
        let mut close_requested = false;

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
                    close_requested = true;
                    runnable.close(&self.context);
                    *control_flow = ControlFlow::Exit;
                }
                Event::Redraw => {
                    let delta = timer.delta();

                    if !close_requested {
                        runnable.update(&self.context, delta);
                        log::trace!("[Redraw] FPS: {}", 1. / delta);
                        match runnable.render(&self.context) {
                            Ok(_) => {}
                            Err(RenderError::Lost | RenderError::Outdated) => {}
                            Err(e) => error!("{:?}", e),
                        }
                    }
                    self.context.frame_number += 1;
                }
                Event::Cleared => {
                    self.context.surface.window.request_redraw();
                }
                Event::Continue => (),
            }
        });
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new("Default", 800, 600)
    }
}

#[allow(async_fn_in_trait)]
pub trait Run: Sized {
    async fn create(context: &Context) -> Self;
    async fn start(&mut self, ctx: &Context);
    fn update(&mut self, ctx: &Context, delta: f32);
    fn render(&mut self, ctx: &Context) -> Result<(), RenderError>;
    fn input(&mut self, ctx: &Context, input: Input);
    fn resize(&mut self, ctx: &Context, width: u32, height: u32);
    fn close(&mut self, ctx: &Context);
}
