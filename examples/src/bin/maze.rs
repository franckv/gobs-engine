use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

use examples::CameraController;
use gobs_game as game;
use gobs_wgpu as render;

use game::{
    app::{Application, Run},
    input::Input,
};
use render::{render::Gfx, scene::Scene};

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let scene = Scene::new(gfx).await;
        let camera_controller = CameraController::new(4.0, 0.4);

        App {
            camera_controller,
            scene,
        }
    }

    fn update(&mut self, delta: f32, gfx: &mut Gfx) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);
        self.scene.update(gfx, delta);
    }

    fn render(&mut self, gfx: &mut Gfx) -> Result<(), wgpu::SurfaceError> {
        gfx.render(&self.scene)
    }

    fn input(&mut self, _gfx: &mut Gfx, input: Input) {
        match input {
            Input::KeyPressed(key) => {
                self.camera_controller.key_pressed(key);
            }
            Input::KeyReleased(key) => {
                self.camera_controller.key_released(key);
            }
            Input::MousePressed => {
                self.camera_controller.mouse_pressed();
            }
            Input::MouseReleased => {
                self.camera_controller.mouse_released();
            }
            Input::MouseWheel(delta) => {
                self.camera_controller.mouse_scroll(delta);
            }
            Input::MouseMotion(dx, dy) => {
                self.camera_controller.mouse_drag(dx, dy);
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32, gfx: &mut Gfx) {
        self.scene.resize(gfx, width, height)
    }
}

fn main() {
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

    Application::new().run::<App>();
}
