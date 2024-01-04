use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

pub fn init_logger() {
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    log::info!("Logger initialized");
}
