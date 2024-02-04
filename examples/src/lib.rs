mod controller;

use env_logger::Builder;

pub use controller::CameraController;
use gobs::{
    game::input::{Input, Key},
    render::{context::Context, graph::FrameGraph},
    scene::scene::Scene,
};

pub fn init_logger() {
    Builder::new().filter_level(log::LevelFilter::Info).init();

    log::info!("Logger initialized");
}

pub fn default_input(
    ctx: &Context,
    scene: &Scene,
    graph: &mut FrameGraph,
    camera_controller: &mut CameraController,
    input: Input,
) {
    log::trace!("Input");

    match input {
        Input::KeyPressed(key) => match key {
            Key::E => graph.render_scaling = (graph.render_scaling + 0.1).min(1.),
            Key::A => graph.render_scaling = (graph.render_scaling - 0.1).max(0.1),
            Key::L => log::info!("{:?}", ctx.allocator.allocator.lock().unwrap()),
            Key::C => log::info!("{:?}", scene.camera),
            _ => camera_controller.key_pressed(key),
        },
        Input::KeyReleased(key) => camera_controller.key_released(key),
        Input::MousePressed => camera_controller.mouse_pressed(),
        Input::MouseReleased => camera_controller.mouse_released(),
        Input::MouseWheel(delta) => camera_controller.mouse_scroll(delta),
        Input::MouseMotion(dx, dy) => camera_controller.mouse_drag(dx, dy),
        _ => (),
    }
}
