use examples::CameraController;
use glam::{Quat, Vec3};
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use scene::light::Light;
use scene::scene::Scene;
use scene::Gfx;
use scene::{
    camera::{Camera, CameraProjection},
    RenderError, ShaderType,
};

const CUBE: &str = "cube.obj";

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let camera = Camera::new(
            (3.0, 4.0, 5.0),
            CameraProjection::new(
                gfx.width(),
                gfx.height(),
                (45.0 as f32).to_radians(),
                0.1,
                150.0,
            ),
            (30.0 as f32).to_radians(),
            (-20.0 as f32).to_radians(),
        );

        let light = Light::new((8.0, 2.0, 8.0), (1., 1., 0.9));

        let mut scene = Scene::new(gfx, camera, light).await;

        let cube = scene
            .load_model(gfx, CUBE, ShaderType::Phong, 1.0)
            .await
            .unwrap();

        scene.add_node(
            scene.light.position,
            Quat::from_axis_angle(Vec3::Z, 0.0),
            cube,
        );

        let camera_controller = CameraController::new(4.0, 0.4);

        App {
            camera_controller,
            scene,
        }
    }

    fn update(&mut self, delta: f32, gfx: &mut Gfx) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        self.scene.update(gfx);
    }

    fn render(&mut self, gfx: &mut Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
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
    examples::init_logger();

    Application::new().run::<App>();
}
