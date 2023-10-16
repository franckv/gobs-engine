use glam::{Quat, Vec3};
use log::*;

use crate::camera::{Camera, CameraController, CameraProjection, CameraResource};
use crate::light::{Light, LightResource};
use crate::model::{Instance, Model, Texture};
use crate::render::Gfx;
use crate::resource;
use crate::scene::Node;
use crate::shader::{PhongShader, SolidShader};

const MAP: &str = include_str!("../../assets/dungeon.map");
const WALL: &str = "cube.obj";
const TREE: &str = "tree.obj";
const LIGHT: &str = "sphere.obj";
const TILE_SIZE: f32 = 2.;

pub struct Scene {
    pub solid_shader: SolidShader,
    pub phong_shader: PhongShader,
    camera: Camera,
    pub camera_resource: CameraResource,
    pub camera_controller: CameraController,
    light: Light,
    pub light_resource: LightResource,
    depth_texture: Texture,
    pub light_model: Model,
    pub nodes: Vec<Node>,
    pub models: Vec<Model>,
    pub instance_buffers: Vec<wgpu::Buffer>,
}

impl Scene {
    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub async fn new(gfx: &Gfx) -> Self {
        let solid_shader = SolidShader::new(&gfx).await;
        let phong_shader = PhongShader::new(&gfx).await;

        let camera_resource = gfx.create_camera_resource(&phong_shader.layouts[0]);

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

        let camera_controller = CameraController::new(4.0, 0.4);

        let light_resource = gfx.create_light_resource(&phong_shader.layouts[1]);
        let light = Light::new((8.0, 2.0, 8.0), (1., 1., 0.9));

        let wall = resource::load_model(WALL, gfx.device(), gfx.queue(), &phong_shader.layouts[2])
            .await
            .unwrap();
        let tree = resource::load_model(TREE, gfx.device(), gfx.queue(), &phong_shader.layouts[2])
            .await
            .unwrap();
        let mut models = Vec::new();
        models.push(wall);
        models.push(tree);

        let nodes: Vec<Node> = Self::load_scene();

        let mut instance_buffers = Vec::new();
        for i in 0..models.len() {
            let instance_data = nodes
                .iter()
                .filter(|n| n.model() == i)
                .map(|n| n.transform().to_raw())
                .collect::<Vec<_>>();
            let instance_buffer = gfx.create_instance_buffer(&instance_data);
            instance_buffers.push(instance_buffer);
        }

        let depth_texture = Texture::create_depth_texture(gfx, "depth_texture");

        let light_model =
            resource::load_model(LIGHT, gfx.device(), gfx.queue(), &phong_shader.layouts[2])
                .await
                .unwrap();

        Scene {
            solid_shader,
            phong_shader,
            camera,
            camera_resource,
            camera_controller,
            light,
            light_resource,
            depth_texture,
            light_model,
            nodes,
            models,
            instance_buffers,
        }
    }

    pub fn resize(&mut self, gfx: &Gfx, width: u32, height: u32) {
        self.depth_texture = Texture::create_depth_texture(gfx, "depth_texture");
        self.camera.projection.resize(width, height);
    }

    pub fn update(&mut self, gfx: &Gfx, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_resource.update(gfx, &self.camera);

        let old_position: Vec3 = self.light.position;
        let position: Vec3 =
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (60.0 * dt).to_radians())
                * old_position)
                .into();

        self.light.update(position);
        self.light_resource.update(&gfx, &self.light);
    }

    pub fn load_scene() -> Vec<Node> {
        info!("Load scene");

        let mut nodes = Vec::new();

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
                    let node = Node::new(Instance { position, rotation }, 0);

                    nodes.push(node);
                }
                't' => {
                    i += TILE_SIZE;
                    let position = Vec3 {
                        x: i - 32.,
                        y: 0.0,
                        z: j - 32.,
                    };
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0.0);
                    let node = Node::new(Instance { position, rotation }, 1);

                    nodes.push(node);
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

        nodes
    }
}
