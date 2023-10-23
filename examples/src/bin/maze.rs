use log::*;

use glam::{Quat, Vec3};

use examples::CameraController;
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use scene::{light::Light, MaterialBuilder, ModelBuilder};
use scene::scene::Scene;
use scene::Gfx;
use scene::{
    camera::{Camera, CameraProjection},
    RenderError, ShaderType,
};
use uuid::Uuid;

struct App {
    camera_controller: CameraController,
    scene: Scene,
    light_model: Uuid,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let camera = Camera::new(
            (0.0, 25.0, 25.0),
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
        let light_position = light.position;

        let mut scene = Scene::new(gfx, camera, light).await;

        let wall_model = ModelBuilder::new()
            .add_mesh(
                scene::shape::Shapes::cube(gfx, ShaderType::Phong.vertex_flags()),
                0,
            )
            .add_material(
                MaterialBuilder::new("diffuse")
                    .diffuse_texture(gfx, examples::WALL_TEXTURE)
                    .await
                    .normal_texture(gfx, examples::WALL_TEXTURE_N)
                    .await
                    .build(gfx, &scene.phong_shader),
            )
            .build();
        
        let id = scene.add_model(wall_model, ShaderType::Phong);

        Self::load_scene(&mut scene, id);

        let light_model = scene
            .load_model(gfx, examples::LIGHT, ShaderType::Solid, 0.3)
            .await
            .unwrap();
        scene.add_node(
            light_position,
            Quat::from_axis_angle(Vec3::Z, 0.0),
            light_model,
        );

        let camera_controller = CameraController::new(3.0, 0.4);

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
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (angular_speed * delta).to_radians())
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
    pub fn load_scene(scene: &mut Scene, wall_model: Uuid) {
        info!("Load scene");

        let offset = 16.;

        let (mut i, mut j) = (0., 0.);

        for c in examples::MAP.chars() {
            match c {
                'w' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: 0.0,
                        z: j - offset,
                    };
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0.0);

                    scene.add_node(position, rotation, wall_model);
                }
                't' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: 0.0,
                        z: j - offset,
                    };
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0.0);

                    scene.add_node(position, rotation, wall_model);
                }
                '.' | '@' => {
                    i += examples::TILE_SIZE;
                }
                '\n' => {
                    j += examples::TILE_SIZE;
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
