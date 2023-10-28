use log::*;

use glam::{Quat, Vec3};

use examples::CameraController;
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use scene::scene::Scene;
use scene::Gfx;
use scene::{
    camera::{Camera, CameraProjection},
    RenderError,
};
use scene::{light::Light, MaterialBuilder, ModelBuilder};
use uuid::Uuid;

struct App {
    camera_controller: CameraController,
    scene: Scene,
    light_model: Uuid,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let camera = Camera::new(
            (0., 0., 0.),
            CameraProjection::new(
                gfx.width(),
                gfx.height(),
                (45. as f32).to_radians(),
                0.1,
                150.,
            ),
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
        );

        let light = Light::new((8., 2., 8.), (1., 1., 0.9));
        let light_position = light.position;

        let mut scene = Scene::new(gfx, camera, light).await;

        let material = MaterialBuilder::new("diffuse")
            .diffuse_texture(gfx, examples::WALL_TEXTURE)
            .await
            .normal_texture(gfx, examples::WALL_TEXTURE_N)
            .await
            .build(gfx, &scene.phong_shader);

        let wall_model = ModelBuilder::new()
            .add_mesh(
                scene::shape::Shapes::cube(
                    gfx,
                    scene.phong_shader.vertex_flags(),
                    3,
                    2,
                    &[5, 5, 5, 5, 6, 4],
                ),
                Some(material.clone()),
            )
            .build();

        let floor_model = ModelBuilder::new()
            .add_mesh(
                scene::shape::Shapes::cube(gfx, scene.phong_shader.vertex_flags(), 3, 2, &[4]),
                Some(material),
            )
            .build();

        let wall_id = scene.add_model(wall_model, scene.phong_shader.clone());
        let floor_id = scene.add_model(floor_model, scene.phong_shader.clone());

        let (pos_x, pos_y, pos_z) = Self::load_scene(&mut scene, wall_id, floor_id);

        scene.camera.position = (pos_x, pos_y, pos_z).into();

        let light_model = scene
            .load_model(gfx, examples::LIGHT, scene.solid_shader.clone(), 0.3)
            .await
            .unwrap();
        scene.add_node(
            light_position,
            Quat::from_axis_angle(Vec3::Z, 0.),
            light_model,
        );

        let camera_controller = CameraController::new(3., 0.4);

        App {
            camera_controller,
            scene,
            light_model,
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

        for node in &mut self.scene.nodes {
            if node.model() == self.light_model {
                node.set_transform(position, node.transform().rotation);
            }
        }

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

impl App {
    pub fn load_scene(scene: &mut Scene, wall_model: Uuid, floor_model: Uuid) -> (f32, f32, f32) {
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

                    scene.add_node(position, rotation, wall_model);
                    position.y = -examples::TILE_SIZE;
                    scene.add_node(position, rotation, floor_model);
                }
                '@' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    (pos_x, pos_z) = (position.x, position.z);
                    scene.add_node(position, rotation, floor_model);
                }
                '.' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    scene.add_node(position, rotation, floor_model);
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
    examples::init_logger();

    Application::new().run::<App>();
}
