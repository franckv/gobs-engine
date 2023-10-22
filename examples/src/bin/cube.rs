use examples::CameraController;
use glam::{Quat, Vec3};
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use scene::Gfx;
use scene::{
    camera::{Camera, CameraProjection},
    RenderError, ShaderType,
};
use scene::{light::Light, ModelBuilder};
use scene::{scene::Scene, MaterialBuilder};

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let camera = Camera::new(
            (-2., 2., 2.),
            CameraProjection::new(
                gfx.width(),
                gfx.height(),
                (45.0 as f32).to_radians(),
                0.1,
                150.0,
            ),
            (-45. as f32).to_radians(),
            (-34. as f32).to_radians(),
        );

        let light = Light::new((0., 0., 10.), (1., 1., 0.9));

        let mut scene = Scene::new(gfx, camera, light).await;

        let cube = ModelBuilder::new()
            .add_mesh(
                scene::shape::Shapes::cube(gfx, ShaderType::Phong.vertex_flags()),
                0,
            )
            .add_material(
                MaterialBuilder::new("diffuse")
                    .diffuse_texture("cube-diffuse.jpg", gfx)
                    .await
                    .normal_texture("cube-normal.png", gfx)
                    .await
                    .build(gfx, &scene.phong_shader.layouts()[2]),
            )
            .build();

        let id = scene.add_model(cube, ShaderType::Phong);

        scene.add_node([0., 0., 0.].into(), Quat::IDENTITY, id);

        let camera_controller = CameraController::new(3.0, 0.4);

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
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (angular_speed * delta).to_radians())
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
