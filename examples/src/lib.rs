mod controller;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

pub use controller::CameraController;

pub fn init_logger() {
    let config_other = ConfigBuilder::new().add_filter_ignore_str("gobs").build();
    let config_self = ConfigBuilder::new().add_filter_allow_str("gobs").add_filter_allow_str("examples").build();

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
