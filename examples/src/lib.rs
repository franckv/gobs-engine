mod controller;

use std::sync::Arc;

use gobs_scene as scene;

use scene::{Gfx, InstanceFlag, PipelineFlag, Shader, VertexFlag};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

pub use controller::CameraController;

pub const MAP: &str = include_str!("../assets/dungeon.map");
pub const CUBE: &str = "cube.obj";
pub const LIGHT: &str = "sphere.obj";
pub const TILE_SIZE: f32 = 1.;
pub const WALL_TEXTURE: &str = "tileset.png";
pub const WALL_TEXTURE_N: &str = "stone_n.png";

pub fn init_logger() {
    let config_other = ConfigBuilder::new()
        .add_filter_ignore_str("examples")
        .build();
    let config_self = ConfigBuilder::new()
        .add_filter_allow_str("examples")
        .build();

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
        PipelineFlag::CULLING | PipelineFlag::DEPTH,
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
        PipelineFlag::CULLING | PipelineFlag::DEPTH,
    )
    .await
}


pub async fn wire_shader(gfx: &Gfx) -> Arc<Shader> {
    Shader::new(
        gfx,
        "Wire",
        "wire.wgsl",
        VertexFlag::POSITION,
        InstanceFlag::MODEL,
        PipelineFlag::CULLING | PipelineFlag::DEPTH | PipelineFlag::LINE,
    )
    .await
}
