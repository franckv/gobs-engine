mod controller;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

pub use controller::CameraController;

pub const MAP: &str = include_str!("../assets/dungeon.map");
pub const CUBE: &str = "cube.obj";
pub const LIGHT: &str = "sphere.obj";
pub const TILE_SIZE: f32 = 1.;
pub const WALL_TEXTURE: &str = "stone.png";
pub const WALL_TEXTURE_N: &str = "stone_n.png";

pub fn init_logger() {
    let config_other = ConfigBuilder::new().add_filter_ignore_str("gobs").build();
    let config_self = ConfigBuilder::new().add_filter_allow_str("gobs").build();

    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            config_other,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        TermLogger::new(
            LevelFilter::Info,
            config_self,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ]);
}
