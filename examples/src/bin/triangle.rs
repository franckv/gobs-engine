use std::sync::Arc;

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
use scene::{camera::Camera, RenderError};
use scene::{light::Light, ModelBuilder};

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
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

        let mut scene = Scene::new(gfx, camera, light, solid_shader.clone()).await;

        let triangle: Arc<scene::Model> = ModelBuilder::new()
            .scale(Vec3::new(300., 300., 1.))
            .add_mesh(
                scene::shape::Shapes::triangle(
                    [1., 0., 0., 0.5],
                    [0., 1., 0., 0.5],
                    [0., 0., 1., 0.5],
                ),
                None,
            )
            .build(gfx, solid_shader);

        scene.add_node("main", [0., 0., 0.].into(), Quat::IDENTITY, triangle);

        let camera_controller = CameraController::new(3., 0.4);

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
