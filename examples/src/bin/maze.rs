use std::sync::Arc;

use log::*;

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

const WALL_LAYER: &str = "wall";
const FLOOR_LAYER: &str = "floor";
const LIGHT_LAYER: &str = "light";

struct App {
    camera_controller: CameraController,
    scene: Scene,
    light_model: Arc<Model>,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
        let camera = Camera::perspective(
            (0., 0., 0.),
            gfx.width() as f32 / gfx.height() as f32,
            (45. as f32).to_radians(),
            0.1,
            150.,
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((8., 2., 8.), Color::new(1., 1., 0.9, 1.));
        let light_position = light.position;

        let phong_shader = examples::phong_shader(gfx).await;
        let solid_shader = examples::solid_shader(gfx).await;
        let wire_shader = examples::wire_shader(gfx).await;

        let mut scene = Scene::new(gfx, camera, light, &[wire_shader.clone()]).await;
        scene.toggle_pass(&wire_shader.name);

        let wall_material = MaterialBuilder::new("diffuse")
            .diffuse_texture(examples::WALL_TEXTURE)
            .await
            .normal_texture(examples::WALL_TEXTURE_N)
            .await
            .build();

        let floor_material = MaterialBuilder::new("diffuse")
            .diffuse_texture(examples::FLOOR_TEXTURE)
            .await
            .normal_texture(examples::FLOOR_TEXTURE_N)
            .await
            .build();

        let wall_model = ModelBuilder::new()
            .add_mesh(Shapes::cube(1, 1, &[1]), Some(wall_material.clone()))
            .build(phong_shader.clone());

        let floor_model = ModelBuilder::new()
            .add_mesh(Shapes::cube(3, 2, &[4]), Some(floor_material))
            .build(phong_shader);

        let (pos_x, pos_y, pos_z) = Self::load_scene(&mut scene, wall_model, floor_model);

        scene.camera.position = (pos_x, pos_y, pos_z).into();

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

impl App {
    fn load_scene(
        scene: &mut Scene,
        wall_model: Arc<Model>,
        floor_model: Arc<Model>,
    ) -> (f32, f32, f32) {
        info!("Load scene");

        let offset = 16.;

        let (mut i, mut j) = (0., 0.);

        let (mut pos_x, pos_y, mut pos_z) = (0., 0., 0.);

        let rotation = Quat::from_axis_angle(Vec3::Z, 0.);

        for c in examples::MAP.chars() {
            match c {
                'w' => {
                    i += examples::TILE_SIZE;
                    let mut position = Vec3 {
                        x: i - offset,
                        y: 0.,
                        z: j - offset,
                    };

                    scene.add_node(
                        WALL_LAYER,
                        position,
                        rotation,
                        Vec3::ONE,
                        wall_model.clone(),
                    );
                    position.y = -examples::TILE_SIZE;
                    scene.add_node(
                        FLOOR_LAYER,
                        position,
                        rotation,
                        Vec3::ONE,
                        floor_model.clone(),
                    );
                }
                '@' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    (pos_x, pos_z) = (position.x, position.z);
                    scene.add_node(
                        FLOOR_LAYER,
                        position,
                        rotation,
                        Vec3::ONE,
                        floor_model.clone(),
                    );
                }
                '.' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    scene.add_node(
                        FLOOR_LAYER,
                        position,
                        rotation,
                        Vec3::ONE,
                        floor_model.clone(),
                    );
                }
                '\n' => {
                    j += examples::TILE_SIZE;
                    i = 0.;
                }
                _ => (),
            }
        }

        (pos_x, pos_y, pos_z)
    }
}
fn main() {
    examples::init_logger(module_path!());

    Application::new().run::<App>();
}
