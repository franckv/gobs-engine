use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::{self, NamedKey},
    window::Window,
};

use gobs_assets::config::GltfConfig;
use gobs_core::{ConfigWriter as _, GobsConfig, Input, logger, utils::timer::Timer};
use gobs_render::{RenderConfig, RenderError, RenderHalConfig};

use crate::{AppError, context::GobsContext};

pub struct Application<R>
where
    R: GobsGame + 'static,
{
    pub runnable: Option<R>,
    pub context: Option<R::Context>,
    pub timer: Timer,
    close_requested: bool,
    is_minimized: bool,
    title: String,
    config: GobsConfig,
    width: u32,
    height: u32,
}

impl<R> ApplicationHandler for Application<R>
where
    R: GobsGame + 'static,
{
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_title(&self.title)
            .with_resizable(true);

        let window = event_loop.create_window(window_attributes).unwrap();

        tracing::info!(target: logger::EVENTS, "Start main loop");

        self.build(Some(window)).block_on();

        self.timer.reset();
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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
                    context.resize();
                    runnable.resize(context, physical_size.width, physical_size.height);
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: key_code,
                            state,
                            text,
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
                                Self::process_input(context, runnable, Input::KeyPressed(key));
                                if let Some(text) = text {
                                    for c in text.chars().filter(|c| !c.is_control()) {
                                        Self::process_input(context, runnable, Input::Char(c));
                                    }
                                }
                            }
                            ElementState::Released => {
                                Self::process_input(context, runnable, Input::KeyReleased(key));
                            }
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    Self::process_input(
                        context,
                        runnable,
                        Input::CursorMoved(position.x, position.y),
                    );
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                            scroll as f32
                        }
                    };
                    Self::process_input(context, runnable, Input::MouseWheel(delta));
                }
                WindowEvent::MouseInput { button, state, .. } => match state {
                    ElementState::Pressed => {
                        Self::process_input(context, runnable, Input::MousePressed(button.into()))
                    }
                    ElementState::Released => {
                        Self::process_input(context, runnable, Input::MouseReleased(button.into()))
                    }
                },
                WindowEvent::RedrawRequested => {
                    let delta = self.timer.delta();

                    if !self.close_requested {
                        if runnable.should_update(context) {
                            context.pre_update(delta);
                            runnable.update(context, delta);
                            context.post_update(delta);
                        }
                        tracing::trace!(target: logger::EVENTS, "[Redraw] FPS: {}", context.fps());
                        if !context.is_minimized() {
                            if self.is_minimized {
                                self.is_minimized = false;
                                context.resize();
                            }
                            match runnable.render(context) {
                                Ok(_) => {}
                                Err(RenderError::Lost | RenderError::Outdated) => {}
                                Err(e) => tracing::error!(target: logger::EVENTS, "{:?}", e),
                            }

                            tracing_tracy::client::frame_mark();
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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
                Self::process_input(context, runnable, Input::MouseMotion(delta.0, delta.1))
            }
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(context) = &mut self.context {
            context.request_redraw();
        }
    }
}

impl<R> Application<R>
where
    R: GobsGame + 'static,
{
    pub fn new(title: &str, width: u32, height: u32) -> Application<R> {
        Application {
            context: None,
            runnable: None,
            close_requested: false,
            is_minimized: false,
            timer: Timer::new(),
            title: title.to_string(),
            config: Self::default_config(),
            width,
            height,
        }
    }

    pub async fn build(&mut self, window: Option<Window>) {
        #[cfg(debug_assertions)]
        let validation_enabled = true;
        #[cfg(not(debug_assertions))]
        let validation_enabled = false;

        tracing::info!("Running with validation layers: {}", validation_enabled);

        let mut context =
            R::Context::new(&self.title, self.config.clone(), window, validation_enabled);

        let mut runnable = R::create(&mut context).await.unwrap();
        runnable.start(&mut context).await;

        self.context = Some(context);
        self.runnable = Some(runnable);
    }

    pub fn with_config<F>(mut self, mut f: F) -> Self
    where
        F: FnMut(&mut GobsConfig),
    {
        f(&mut self.config);

        self
    }

    pub fn default_config() -> GobsConfig {
        let mut config = GobsConfig::default();
        config.register::<RenderConfig>();
        config.register::<RenderHalConfig>();
        config.register::<GltfConfig>();

        config
    }

    pub fn run_once<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut R, &mut R::Context),
    {
        if let (Some(runnable), Some(context)) = (&mut self.runnable, &mut self.context) {
            f(runnable, context);
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }

    fn process_input(context: &mut R::Context, runnable: &mut R, input: Input) {
        context.input(input);
        runnable.input(context, input);
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

#[allow(async_fn_in_trait)]
pub trait GobsGame: Sized {
    type Context: GobsContext;

    async fn create(ctx: &mut Self::Context) -> Result<Self, AppError>;
    async fn start(&mut self, ctx: &mut Self::Context);
    fn update(&mut self, ctx: &mut Self::Context, delta: f32);
    fn should_update(&mut self, _ctx: &mut Self::Context) -> bool {
        true
    }
    fn render(&mut self, ctx: &mut Self::Context) -> Result<(), RenderError>;
    fn input(&mut self, ctx: &mut Self::Context, input: Input);
    fn resize(&mut self, ctx: &mut Self::Context, width: u32, height: u32);
    fn close(&mut self, ctx: &mut Self::Context);
}
