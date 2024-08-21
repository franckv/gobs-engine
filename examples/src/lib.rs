mod app;
mod controller;
mod ui;

use tracing::Level;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

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
pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;

pub fn init_logger() {
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::CLOSE)
        .finish();
    tracing::subscriber::set_global_default(sub).unwrap();

    tracing::info!("Logger initialized");
}
