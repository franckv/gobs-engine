mod app;
mod controller;

use env_logger::Builder;

use gobs::{
    game::input::{Input, Key},
    render::{context::Context, graph::FrameGraph},
    scene::scene::Scene,
};

pub use app::SampleApp;
pub use controller::CameraController;

pub const WALL_TEXTURE: &str = "wall.png";
pub const WALL_TEXTURE_N: &str = "wall_n.png";
pub const ATLAS: &[&str] = &[
    "blocks/dirt.png",
    "blocks/stone.png",
    "blocks/grass_side.png",
    "blocks/grass_top.png",
    "blocks/cobblestone.png",
    "blocks/mossy_cobblestone.png",
];
pub const ATLAS_N: &[&str] = &[
    "blocks/dirt_n.png",
    "blocks/stone_n.png",
    "blocks/grass_side_n.png",
    "blocks/grass_top_n.png",
    "blocks/cobblestone_n.png",
    "blocks/mossy_cobblestone_n.png",
];
pub const ATLAS_COLS: u32 = 3;
pub const ATLAS_ROWS: u32 = 2;
pub const MAP: &str = include_str!("../assets/dungeon.map");
pub const TILE_SIZE: f32 = 1.;

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
