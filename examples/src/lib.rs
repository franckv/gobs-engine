mod controller;
mod ui;

use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{
    EnvFilter, Layer, filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt as _,
};
use tracing_tracy::TracyLayer;

use gobs::core::ImageFormat;

pub use controller::InputManager;
pub use ui::Ui;

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
pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;
pub const GLTF_MODEL: &str = "house2.glb";
pub const DIFFUSE_FORMAT: ImageFormat = ImageFormat::R8g8b8a8Srgb;
pub const NORMAL_FORMAT: ImageFormat = ImageFormat::R8g8b8a8Unorm;

pub fn init_logger() {
    tracing_subscriber::registry()
        .with(
            fmt::layer().with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            ),
        )
        .with(
            TracyLayer::default()
                .with_filter(Targets::default().with_target("profile", Level::TRACE)),
        )
        .init();

    tracing::info!(
        "Logger initialized (RUST_LOG={:?})",
        std::env::var("RUST_LOG")
    );
}
