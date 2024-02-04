mod controller;

use env_logger::Builder;

pub use controller::CameraController;

pub fn init_logger() {
    Builder::new().filter_level(log::LevelFilter::Info).init();

    log::info!("Logger initialized");
}
