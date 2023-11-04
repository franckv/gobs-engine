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
use scene::{camera::Camera, RenderError};

const MODEL_LAYER: &str = "model";

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let camera = Camera::perspective(
            (-2., 2., 5.),
            gfx.width() as f32 / gfx.height() as f32,
            (45. as f32).to_radians(),
            0.1,
            150.,
            (-65. as f32).to_radians(),
            (-20. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((8., 2., 8.), (1., 1., 0.9));

        let shader = examples::phong_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, shader.clone()).await;

        let cube = scene
            .load_model(gfx, examples::CUBE, shader, Vec3::splat(1.))
            .await
            .unwrap();

        scene.add_node(MODEL_LAYER, [0., 0., 0.].into(), Quat::IDENTITY, cube);

        let camera_controller = CameraController::new(3., 0.4);

        App {
            camera_controller,
            scene,
        }
    }

    fn update(&mut self, delta: f32, gfx: &mut Gfx) {
        let angular_speed = 40.;

        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        let old_position: Vec3 = self.scene.light.position;
        let position: Vec3 =
            (Quat::from_axis_angle((0., 1., 0.).into(), (angular_speed * delta).to_radians())
                * old_position)
                .into();

        self.scene.light.update(position);

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
            _ => (),
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
