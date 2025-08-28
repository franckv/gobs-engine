use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::{self, NamedKey},
    window::Window,
};

use gobs_core::{Input, logger, utils::timer::Timer};
use gobs_render::{Display, RenderError};

use crate::{AppError, context::GameContext, options::GameOptions};

pub struct Application<R>
where
    R: Run + 'static,
{
    pub runnable: Option<R>,
    pub context: Option<GameContext>,
    pub timer: Timer,
    close_requested: bool,
    is_minimized: bool,
    title: String,
    options: GameOptions,
    width: u32,
    height: u32,
}

impl<R> ApplicationHandler for Application<R>
where
    R: Run + 'static,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_title(&self.title)
            .with_resizable(true);

        let window = event_loop.create_window(window_attributes).unwrap();

        #[cfg(debug_assertions)]
        let validation_enabled = true;
        #[cfg(not(debug_assertions))]
        let validation_enabled = false;

        tracing::info!("Running with validation layers: {}", validation_enabled);

        let mut context =
            GameContext::new(&self.title, &self.options, Some(window), validation_enabled).unwrap();
        tracing::info!(target: logger::EVENTS, "Start main loop");

        let future = async {
            let mut runnable = R::create(&mut context).await.unwrap();
            runnable.start(&mut context).await;

            runnable
        };

        let runnable = future.block_on();

        self.context = Some(context);
        self.runnable = Some(runnable);
        self.timer.reset();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(runnable) = &mut self.runnable
            && let Some(context) = &mut self.context
        {
            tracing::trace!(target: logger::EVENTS, "evt={:?}", event);

            match event {
                WindowEvent::CloseRequested => {
                    tracing::info!(target: logger::EVENTS, "Stopping");
                    self.close_requested = true;
                }
                WindowEvent::Resized(physical_size) => {
                    tracing::trace!(target: logger::EVENTS,
                        "Resize to : {}/{}",
                        physical_size.width,
                        physical_size.height
                    );
                    context.resize(physical_size.width, physical_size.height);
                    runnable.resize(context, physical_size.width, physical_size.height);
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: key_code,
                            state,
                            ..
                        },
                    ..
                } => match key_code {
                    keyboard::Key::Named(NamedKey::Escape) => {
                        tracing::info!(target: logger::EVENTS, "Stopping");
                        self.close_requested = true;
                    }
                    _ => {
                        let key = key_code.into();
                        match state {
                            ElementState::Pressed => {
                                runnable.input(context, Input::KeyPressed(key))
                            }
                            ElementState::Released => {
                                runnable.input(context, Input::KeyReleased(key))
                            }
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    runnable.input(context, Input::CursorMoved(position.x, position.y));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                            scroll as f32
                        }
                    };
                    runnable.input(context, Input::MouseWheel(delta));
                }
                WindowEvent::MouseInput { button, state, .. } => match state {
                    ElementState::Pressed => {
                        runnable.input(context, Input::MousePressed(button.into()))
                    }
                    ElementState::Released => {
                        runnable.input(context, Input::MouseReleased(button.into()))
                    }
                },
                WindowEvent::RedrawRequested => {
                    let delta = self.timer.delta();

                    if !self.close_requested {
                        if runnable.should_update(context) {
                            context.update(delta);
                            runnable.update(context, delta);
                        }
                        tracing::trace!(target: logger::EVENTS, "[Redraw] FPS: {}", 1. / delta);
                        if !context.renderer.gfx.display.is_minimized() {
                            if self.is_minimized {
                                self.is_minimized = false;
                                context
                                    .renderer
                                    .gfx
                                    .display
                                    .resize(&context.renderer.gfx.device);
                            }
                            match runnable.render(context) {
                                Ok(_) => {}
                                Err(RenderError::Lost | RenderError::Outdated) => {}
                                Err(e) => tracing::error!(target: logger::EVENTS, "{:?}", e),
                            }
                        } else {
                            self.is_minimized = true;
                        }
                    }
                }
                _ => (),
            }
        }

        if self.close_requested {
            self.close();
            event_loop.exit();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(runnable) = &mut self.runnable
            && let Some(context) = &mut self.context
        {
            tracing::trace!(target: logger::EVENTS, "evt={:?}", event);

            if let DeviceEvent::MouseMotion { delta } = event {
                runnable.input(context, Input::MouseMotion(delta.0, delta.1))
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(context) = &mut self.context {
            context.renderer.gfx.request_redraw();
        }
    }
}

impl<R> Application<R>
where
    R: Run + 'static,
{
    pub fn new(title: &str, options: GameOptions, width: u32, height: u32) -> Application<R> {
        Application {
            context: None,
            runnable: None,
            close_requested: false,
            is_minimized: false,
            timer: Timer::new(),
            title: title.to_string(),
            options,
            width,
            height,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }

    pub fn close(&mut self) {
        if let Some(runnable) = &mut self.runnable
            && let Some(context) = &mut self.context
        {
            runnable.close(context);
            context.close();
        }
    }
}

impl<R> Default for Application<R>
where
    R: Run + 'static,
{
    fn default() -> Self {
        Self::new("Default", GameOptions::default(), 800, 600)
    }
}

#[allow(async_fn_in_trait)]
pub trait Run: Sized {
    async fn create(context: &mut GameContext) -> Result<Self, AppError>;
    async fn start(&mut self, ctx: &mut GameContext);
    fn update(&mut self, ctx: &mut GameContext, delta: f32);
    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        true
    }
    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError>;
    fn input(&mut self, ctx: &mut GameContext, input: Input);
    fn resize(&mut self, ctx: &mut GameContext, width: u32, height: u32);
    fn close(&mut self, ctx: &mut GameContext);
}
