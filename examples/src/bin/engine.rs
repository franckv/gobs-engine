use gobs::game::{
    app::{Application, RenderError, Run},
    input::{Input, Key},
};

struct App {
}

impl Run for App {
    async fn create() -> Self {
        log::info!("Create");

        App {}
    }

    fn update(&mut self, delta: f32) {
        log::debug!("Update");
    }

    fn render(&mut self) -> Result<(), RenderError> {
        log::debug!("Render");

        Ok(())
    }

    fn input(&mut self, input: Input) {
        log::debug!("Input");
    }

    fn resize(&mut self, width: u32, height: u32) {
        log::info!("Resize");
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new().run::<App>();
}