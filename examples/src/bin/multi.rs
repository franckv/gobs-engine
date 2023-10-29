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
    RenderError,
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
            (-4., 10., 7.),
            CameraProjection::new(
                gfx.width(),
                gfx.height(),
                (45. as f32).to_radians(),
                0.1,
                150.,
            ),
            (-65. as f32).to_radians(),
            (-50. as f32).to_radians(),
        );

        let light = Light::new((8., 2., 8.), (1., 1., 0.9));

        let mut scene = Scene::new(gfx, camera, light).await;

        let model = scene
            .load_model(gfx, examples::CUBE, scene.phong_shader.clone(), 1.)
            .await
            .unwrap();

        let triangle = ModelBuilder::new()
            .add_mesh(
                scene::shape::Shapes::triangle(
                    gfx,
                    scene.solid_shader.vertex_flags(),
                    [1., 0., 0.],
                    [0., 1., 0.],
                    [0., 0., 1.],
                ),
                None,
            )
            .build();

        let material = MaterialBuilder::new("diffuse")
            .diffuse_texture(gfx, examples::WALL_TEXTURE)
            .await
            .normal_texture(gfx, examples::WALL_TEXTURE_N)
            .await
            .build(gfx, &scene.phong_shader);

        let cube = ModelBuilder::new()
            .add_mesh(
                scene::shape::Shapes::cube(
                    gfx,
                    scene.phong_shader.vertex_flags(),
                    3,
                    2,
                    &[5, 5, 5, 5, 6, 4],
                ),
                Some(material),
            )
            .build();

        scene.add_node(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            model,
            scene.phong_shader.clone(),
        );

        scene.add_node(
            [-3., 0., -3.].into(),
            Quat::IDENTITY,
            triangle,
            scene.solid_shader.clone(),
        );

        scene.add_node(
            [5., 0., 0.].into(),
            Quat::IDENTITY,
            cube,
            scene.phong_shader.clone(),
        );

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
