mod app;
mod controller;
mod ui;

use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

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
pub const MAP: &str = "dungeon.map";
pub const TILE_SIZE: f32 = 1.;
pub const WIDTH: u32 = 1920;
pub const HEIGHT: u32 = 1080;

pub fn init_logger() {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    tracing::info!("Logger initialized");
}
