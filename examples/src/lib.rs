mod controller;

use std::sync::Arc;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

use gobs::core::entity::instance::InstanceFlag;
use gobs::core::geometry::vertex::VertexFlag;
use gobs::scene::{Gfx, PipelineFlag, Shader};

pub use controller::CameraController;

pub const MAP: &str = include_str!("../assets/dungeon.map");
pub const CUBE: &str = "cube.obj";
pub const LIGHT: &str = "sphere.obj";
pub const TILE_SIZE: f32 = 1.;
pub const WALL_TEXTURE: &str = "wall.png";
pub const WALL_TEXTURE_N: &str = "wall_n.png";
pub const FLOOR_TEXTURE: &str = "floor.png";
pub const FLOOR_TEXTURE_N: &str = "floor_n.png";
pub const WIRE_PASS: &str = "Wire";

pub fn init_logger(path: &str) {
    let config_other = ConfigBuilder::new()
        .add_filter_allow_str("gobs")
        .build();
    let config_self = ConfigBuilder::new()
        .add_filter_allow(path.to_string())
        .build();

    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config_other,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        TermLogger::new(
            LevelFilter::Debug,
            config_self,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ]);

    log::debug!("Logger initialized");
}

pub async fn ui_shader(gfx: &Gfx) -> Arc<Shader> {
    Shader::new(
        gfx,
        "UI",
        "ui.wgsl",
        VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE,
        InstanceFlag::MODEL,
        PipelineFlag::ALPHA,
    )
    .await
}

pub async fn phong_shader(gfx: &Gfx) -> Arc<Shader> {
    Shader::new(
        gfx,
        "Phong",
        "phong.wgsl",
        VertexFlag::POSITION | VertexFlag::TEXTURE | VertexFlag::NORMAL,
        InstanceFlag::MODEL | InstanceFlag::NORMAL,
        PipelineFlag::CULLING | PipelineFlag::DEPTH_TEST | PipelineFlag::DEPTH_WRITE,
    )
    .await
}

pub async fn solid_shader(gfx: &Gfx) -> Arc<Shader> {
    Shader::new(
        gfx,
        "Solid",
        "solid.wgsl",
        VertexFlag::POSITION | VertexFlag::COLOR,
        InstanceFlag::MODEL,
        PipelineFlag::CULLING | PipelineFlag::DEPTH_TEST | PipelineFlag::DEPTH_WRITE,
    )
    .await
}

pub async fn wire_shader(gfx: &Gfx) -> Arc<Shader> {
    Shader::new(
        gfx,
        WIRE_PASS,
        "wire.wgsl",
        VertexFlag::POSITION,
        InstanceFlag::MODEL,
        PipelineFlag::CULLING | PipelineFlag::DEPTH_TEST | PipelineFlag::LINE | PipelineFlag::ALPHA,
    )
    .await
}
