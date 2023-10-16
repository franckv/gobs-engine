use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

use gobs_wgpu::app::Application;

fn main() {
    let config_other = ConfigBuilder::new()
        .add_filter_ignore_str(module_path!())
        .build();
    let config_self = ConfigBuilder::new()
        .add_filter_allow_str(module_path!())
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

    pollster::block_on(run());
}

async fn run() {
    let app = Application::new().await;

    app.run();
}
