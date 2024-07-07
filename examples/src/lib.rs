mod app;
mod controller;
mod ui;

use env_logger::Builder;

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
