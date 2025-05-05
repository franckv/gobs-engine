use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::{self, NamedKey},
    window::Window,
};

use gobs_core::{Input, utils::timer::Timer};
use gobs_render::{Display, RenderError};

use crate::{AppError, context::GameContext};

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

        let mut context = GameContext::new(&self.title, Some(window), validation_enabled).unwrap();
        tracing::info!("Start main loop");

        let future = async {
            let mut runnable = R::create(&context).await.unwrap();
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
        if let Some(runnable) = &mut self.runnable {
            if let Some(context) = &mut self.context {
                tracing::trace!("evt={:?}", event);

                match event {
                    WindowEvent::CloseRequested => {
                        tracing::info!("Stopping");
                        self.close_requested = true;
                        runnable.close(context);
                        event_loop.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        tracing::debug!(
                            "Resize to : {}/{}",
                            physical_size.width,
                            physical_size.height
                        );
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
                            tracing::info!("Stopping");
                            self.close_requested = true;
                            runnable.close(context);
                            event_loop.exit();
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
                            MouseScrollDelta::PixelDelta(PhysicalPosition {
                                y: scroll, ..
                            }) => scroll as f32,
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
                            runnable.update(context, delta);
                            tracing::trace!("[Redraw] FPS: {}", 1. / delta);
                            if !context.gfx.display.is_minimized() {
                                if self.is_minimized {
                                    self.is_minimized = false;
                                    context.gfx.display.resize(&context.gfx.device);
                                }
                                match runnable.render(context) {
                                    Ok(_) => {}
                                    Err(RenderError::Lost | RenderError::Outdated) => {}
                                    Err(e) => tracing::error!("{:?}", e),
                                }
                            } else {
                                self.is_minimized = true;
                            }
                        }
                        context.gfx.new_frame();
                    }
                    _ => (),
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(runnable) = &mut self.runnable {
            if let Some(context) = &mut self.context {
                tracing::trace!("evt={:?}", event);

                if let DeviceEvent::MouseMotion { delta } = event {
                    runnable.input(context, Input::MouseMotion(delta.0, delta.1))
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(context) = &mut self.context {
            context.gfx.request_redraw();
        }
    }
}

impl<R> Application<R>
where
    R: Run + 'static,
{
    pub fn new(title: &str, width: u32, height: u32) -> Application<R> {
        Application {
            context: None,
            runnable: None,
            close_requested: false,
            is_minimized: false,
            timer: Timer::new(),
            title: title.to_string(),
            width,
            height,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }
}

impl<R> Default for Application<R>
where
    R: Run + 'static,
{
    fn default() -> Self {
        Self::new("Default", 800, 600)
    }
}

#[allow(async_fn_in_trait)]
pub trait Run: Sized {
    async fn create(context: &GameContext) -> Result<Self, AppError>;
    async fn start(&mut self, ctx: &mut GameContext);
    fn update(&mut self, ctx: &mut GameContext, delta: f32);
    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError>;
    fn input(&mut self, ctx: &GameContext, input: Input);
    fn resize(&mut self, ctx: &mut GameContext, width: u32, height: u32);
    fn close(&mut self, ctx: &GameContext);
}
