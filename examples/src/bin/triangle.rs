use std::sync::Arc;

use glam::{Quat, Vec3};

use gobs::core::entity::{camera::Camera, light::Light};
use gobs::game::{
    app::{Application, Run},
    input::{Input, Key},
};
use gobs::scene::shape::Shapes;
use gobs::scene::{Gfx, Model, ModelBuilder, RenderError, Scene};

use examples::CameraController;

const TRIANGLE_LAYER: &str = "triangle";

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
        let camera = Camera::ortho(
            (0., 0., 10.),
            gfx.width() as f32,
            gfx.height() as f32,
            0.1,
            100.,
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((0., 0., 10.), (1., 1., 1.));

        let solid_shader = examples::solid_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, &[]).await;

        let triangle: Arc<Model> = ModelBuilder::new()
            .add_mesh(
                Shapes::triangle([1., 0., 0., 0.5], [0., 1., 0., 0.5], [0., 0., 1., 0.5]),
                None,
            )
            .build(solid_shader);

        scene.add_node(
            TRIANGLE_LAYER,
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [300., 300., 1.].into(),
            triangle,
        );

        let camera_controller = CameraController::new(3., 0.4);

        App {
            camera_controller,
            scene,
        }
    }

    fn update(&mut self, delta: f32, gfx: &Gfx) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

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
