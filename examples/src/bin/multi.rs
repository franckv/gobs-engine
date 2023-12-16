use std::sync::Arc;

use glam::{Quat, Vec3};

use gobs::core::entity::{camera::Camera, light::Light};
use gobs::core::Color;
use gobs::game::{
    app::{Application, Run},
    input::{Input, Key},
};
use gobs::material::MaterialBuilder;
use gobs::scene::shape::Shapes;
use gobs::scene::{Gfx, Model, ModelBuilder, RenderError, Scene};

use examples::CameraController;

const CUBE_LAYER: &str = "cube";
const TRIANGLE_LAYER: &str = "triangle";
const MODEL_LAYER: &str = "model";
const LIGHT_LAYER: &str = "light";

struct App {
    camera_controller: CameraController,
    scene: Scene,
    light_model: Arc<Model>,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
        let camera = Camera::perspective(
            (-4., 10., 7.),
            gfx.width() as f32 / gfx.height() as f32,
            (45. as f32).to_radians(),
            0.1,
            150.,
            (-65. as f32).to_radians(),
            (-50. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((4., 2., 4.), Color::new(1., 1., 0.9, 1.));
        let light_position = light.position;

        let phong_shader = examples::phong_shader(gfx).await;
        let solid_shader = examples::solid_shader(gfx).await;
        let wire_shader = examples::wire_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, &[wire_shader.clone()]).await;
        scene.toggle_pass(&wire_shader.name);

        let triangle = ModelBuilder::new()
            .add_mesh(
                Shapes::triangle([1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]),
                None,
            )
            .build(solid_shader.clone());

        let material = MaterialBuilder::new("diffuse")
            .diffuse_texture(examples::WALL_TEXTURE)
            .await
            .normal_texture(examples::WALL_TEXTURE_N)
            .await
            .build();

        let model = scene
            .load_model(examples::CUBE, Some(material.clone()), phong_shader.clone())
            .await
            .unwrap();

        let cube = ModelBuilder::new()
            .add_mesh(Shapes::cube(1, 1, &[1]), Some(material))
            .build(phong_shader);

        let light_model = scene
            .load_model(examples::LIGHT, None, solid_shader)
            .await
            .unwrap();

        scene.add_node(
            LIGHT_LAYER,
            light_position,
            Quat::from_axis_angle(Vec3::Z, 0.),
            Vec3::splat(0.3),
            light_model.clone(),
        );

        scene.add_node(
            MODEL_LAYER,
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            Vec3::ONE,
            model,
        );

        scene.add_node(
            TRIANGLE_LAYER,
            [-3., 0., -3.].into(),
            Quat::IDENTITY,
            Vec3::ONE,
            triangle,
        );

        scene.add_node(
            CUBE_LAYER,
            [5., 0., 0.].into(),
            Quat::IDENTITY,
            Vec3::ONE,
            cube,
        );

        let camera_controller = CameraController::new(3., 0.4);

        App {
            camera_controller,
            scene,
            light_model,
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

        for node in self.scene.layer_mut(LIGHT_LAYER).nodes_mut() {
            if node.model().id == self.light_model.id {
                node.move_to_position(position);
            }
        }

        self.scene.update(gfx);
    }

    fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    fn input(&mut self, _gfx: &Gfx, input: Input) {
        match input {
            Input::KeyPressed(key) => match key {
                Key::T => self.scene.toggle_layer(TRIANGLE_LAYER),
                Key::M => self.scene.toggle_layer(MODEL_LAYER),
                Key::C => self.scene.toggle_layer(CUBE_LAYER),
                Key::L => self.scene.toggle_layer(LIGHT_LAYER),
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
    examples::init_logger(module_path!());

    Application::new().run::<App>();
}
