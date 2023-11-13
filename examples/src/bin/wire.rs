use examples::CameraController;
use glam::{Quat, Vec3};
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use scene::scene::Scene;
use scene::Gfx;
use scene::{Camera, RenderError};
use scene::{Light, ModelBuilder};

const CUBE_LAYER: &str = "cube";

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
        let camera = Camera::perspective(
            (-2., 2., 2.),
            gfx.width() as f32 / gfx.height() as f32,
            (45. as f32).to_radians(),
            0.1,
            150.,
            (-45. as f32).to_radians(),
            (-34. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((10., 0., 7.), (1., 1., 0.9));

        let shader = examples::wire_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, &[]).await;

        let cube = ModelBuilder::new()
            .add_mesh(scene::shape::Shapes::cube(3, 2, &[1]), None)
            .build(shader);

        scene.add_node(CUBE_LAYER, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE, cube);

        let camera_controller = CameraController::new(3., 0.4);

        App {
            camera_controller,
            scene,
        }
    }

    fn update(&mut self, delta: f32, gfx: &Gfx) {
        let angular_speed = 40.;

        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        let rot_delta =
            Quat::from_axis_angle((0., 1., 0.).into(), (angular_speed * delta).to_radians());
        let rot_delta_model = Quat::from_axis_angle(
            (0., 1., 0.).into(),
            (0.1 * angular_speed * delta).to_radians(),
        );

        let old_position: Vec3 = self.scene.light.position;
        let position: Vec3 = (rot_delta * old_position).into();

        self.scene.light.update(position);

        for node in self.scene.layer_mut(CUBE_LAYER).nodes_mut() {
            node.rotate(rot_delta_model);
        }

        self.scene.update(gfx);
    }

    fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    fn input(&mut self, _gfx: &Gfx, input: Input) {
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

    fn resize(&mut self, width: u32, height: u32, _gfx: &Gfx) {
        self.scene.resize(width, height)
    }
}

fn main() {
    examples::init_logger();

    Application::new().run::<App>();
}
