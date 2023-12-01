use std::sync::Arc;

use glam::{Quat, Vec3};
use log::info;

use gobs::core::entity::{camera::Camera, light::Light};
use gobs::game::{
    app::{Application, Run},
    input::{Input, Key},
};
use gobs::scene::shape::Shapes;
use gobs::scene::{Gfx, MaterialBuilder, Model, ModelBuilder, RenderError, Scene};

use examples::CameraController;

const IMAGE_LAYER: &str = "fb";

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
        let (width, height) = (gfx.width(), gfx.height());

        let camera = Camera::ortho(
            (0., 0., 1.),
            width as f32,
            height as f32,
            0.1,
            100.,
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        info!("{}/{}", gfx.width(), gfx.height());

        let light = Light::new((0., 0., 10.), (1., 1., 1.));

        let shader = examples::ui_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, &[]).await;

        let framebuffer = Self::generate_framebuffer(width, height);

        let material = MaterialBuilder::new("diffuse")
            .diffuse_buffer(&framebuffer, width, height)
            .await
            .build();

        let image: Arc<Model> = ModelBuilder::new()
            .add_mesh(Shapes::quad(), Some(material))
            .build(shader);

        scene.add_node(
            IMAGE_LAYER,
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [gfx.width() as f32, gfx.height() as f32, 1.].into(),
            image,
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
            _ => (),
        }
    }

    fn resize(&mut self, width: u32, height: u32, _gfx: &Gfx) {
        self.scene.resize(width, height)
    }
}

impl App {
    fn generate_framebuffer(width: u32, height: u32) -> Vec<u8> {
        let mut buffer = Vec::new();

        let border = 50;

        for i in 0..height {
            for j in 0..width {
                //if i < border || i >= height - border || j < border || j >= width - border {
                if i < border && j < border {
                    buffer.push(0);
                    buffer.push(0);
                    buffer.push(255);
                } else {
                    buffer.push(255);
                    buffer.push(0);
                    buffer.push(0);
                }

                buffer.push(255);
            }
        }
        buffer
    }
}

fn main() {
    examples::init_logger();

    Application::new().run::<App>();
}
