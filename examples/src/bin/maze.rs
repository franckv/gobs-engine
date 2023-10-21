use log::*;

use glam::{Quat, Vec3};

use examples::CameraController;
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use scene::{camera::{Camera, CameraProjection}, ShaderType, RenderError};
use scene::light::Light;
use scene::model::Instance;
use scene::node::Node;
use scene::scene::Scene;
use scene::Gfx;
use uuid::Uuid;

const LIGHT: &str = "sphere.obj";
const WALL: &str = "cube.obj";
const TREE: &str = "tree.obj";
const MAP: &str = include_str!("../../assets/dungeon.map");
const TILE_SIZE: f32 = 2.;

struct App {
    camera_controller: CameraController,
    scene: Scene,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let camera = Camera::new(
            (0.0, 50.0, 50.0),
            CameraProjection::new(
                gfx.width(),
                gfx.height(),
                (45.0 as f32).to_radians(),
                0.1,
                150.0,
            ),
            (-90.0 as f32).to_radians(),
            (-50.0 as f32).to_radians(),
        );

        let light = Light::new((8.0, 2.0, 8.0), (1., 1., 0.9));

        let mut scene = Scene::new(gfx, camera, light).await;
        let wall_model = scene
            .load_model(gfx, WALL, ShaderType::Phong)
            .await
            .unwrap();
        scene
            .load_model(gfx, TREE, ShaderType::Phong)
            .await
            .unwrap();
        Self::load_scene(&mut scene, wall_model);

        scene
            .load_model(gfx, LIGHT, ShaderType::Solid)
            .await
            .unwrap();

        let camera_controller = CameraController::new(4.0, 0.4);

        App {
            camera_controller,
            scene,
        }
    }

    fn update(&mut self, delta: f32, gfx: &mut Gfx) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        let old_position: Vec3 = self.scene.light.position;
        let position: Vec3 =
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (10. * delta).to_radians())
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

impl App {
    pub fn load_scene(scene: &mut Scene, wall_model: Uuid) {
        info!("Load scene");

        let (mut i, mut j) = (0., 0.);

        for c in MAP.chars() {
            match c {
                'w' => {
                    i += TILE_SIZE;
                    let position = Vec3 {
                        x: i - 32.,
                        y: 0.0,
                        z: j - 32.,
                    };
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0.0);
                    let node = Node::new(Instance { position, rotation }, wall_model);

                    scene.add_node(node);
                }
                't' => {
                    i += TILE_SIZE;
                    let position = Vec3 {
                        x: i - 32.,
                        y: 0.0,
                        z: j - 32.,
                    };
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0.0);
                    let node = Node::new(Instance { position, rotation }, wall_model);

                    scene.add_node(node);
                }
                '.' | '@' => {
                    i += TILE_SIZE;
                }
                '\n' => {
                    j += TILE_SIZE;
                    i = 0.;
                }
                _ => (),
            }
        }
    }
}
fn main() {
    examples::init_logger();

    Application::new().run::<App>();
}
