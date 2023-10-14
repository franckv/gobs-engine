use log::*;
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, ColorChoice, TerminalMode};
use winit::event::*;
use winit::event_loop::*;
use winit::window::WindowBuilder;

use wgpu_test::State;

use gobs_utils::timer::Timer;

pub async fn run() {
    let config_other = ConfigBuilder::new().add_filter_ignore_str(module_path!()).build();
    let config_self = ConfigBuilder::new().add_filter_allow_str(module_path!()).build();

    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, config_other, TerminalMode::Mixed, ColorChoice::Auto),
            TermLogger::new(LevelFilter::Info, config_self, TerminalMode::Mixed, ColorChoice::Auto)
        ]
    );

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;
    let mut timer = Timer::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta },
                ..
            } => if state.mouse_pressed {
                state.mouse_input(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update(timer.delta());
                match state.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => state.resize(*state.size()),
                    Err(e) => error!("{:?}", e)
                }
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    pollster::block_on(run());
}
