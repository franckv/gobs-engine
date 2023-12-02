use glam::{Quat, Vec3};

use gobs::core::entity::{camera::Camera, light::Light};
use gobs::core::Color;
use gobs::game::{
    app::{Application, Run},
    input::{Input, Key},
};
use gobs::scene::{Gfx, MaterialBuilder, RenderError, Scene};

use examples::CameraController;

const MODEL_LAYER: &str = "model";

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
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

        let light = Light::new((8., 2., 8.), Color::new(1., 1., 0.9, 1.));

        let shader = examples::phong_shader(gfx).await;
        let wire = examples::wire_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, &[wire.clone()]).await;
        scene.toggle_pass(&wire.name);

        let material = MaterialBuilder::new("diffuse")
            .diffuse_texture(examples::WALL_TEXTURE)
            .await
            .normal_texture(examples::WALL_TEXTURE_N)
            .await
            .build();

        let cube = scene
            .load_model(examples::CUBE, Some(material), shader)
            .await
            .unwrap();

        scene.add_node(
            MODEL_LAYER,
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            Vec3::ONE,
            cube,
        );

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

        let old_position = self.scene.light.position;
        let position =
            Quat::from_axis_angle((0., 1., 0.).into(), (angular_speed * delta).to_radians())
                * old_position;

        self.scene.light.update(position);

        self.scene.update(gfx);
    }

    fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    fn input(&mut self, _gfx: &Gfx, input: Input) {
        match input {
            Input::KeyPressed(key) => match key {
                Key::W => self.scene.toggle_pass(examples::WIRE_PASS),
                _ => self.camera_controller.key_pressed(key),
            },
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
