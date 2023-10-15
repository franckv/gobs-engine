use glam::{ Quat, Vec3 };
use log::*;

use crate::Camera;
use crate::CameraController;
use crate::camera::CameraProjection;
use crate::camera::CameraResource;
use crate::Gfx;
use crate::Instance;
use crate::Light;
use crate::light::LightResource;
use crate::pass::{ LightPass, ModelPass };
use crate::resource;
use crate::model::{ Model, Texture };

const TILE_SIZE: f32 = 2.;

pub struct Scene {
    pub light_pass: LightPass,
    pub model_pass: ModelPass,
    camera: Camera,
    pub camera_resource: CameraResource,
    pub camera_controller: CameraController,
    light: Light,
    pub light_resource: LightResource,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    pub obj_model: Model,
}

impl Scene {
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_controller(&self) -> &CameraController {
        &self.camera_controller
    }

    pub fn light(&self) -> &Light {
        &self.light
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }

    pub fn instance_buffer(&self) -> &wgpu::Buffer {
        &self.instance_buffer
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn obj_model(&self) -> &Model {
        &self.obj_model
    }

    pub async fn new(gfx: &Gfx) -> Self {
        let light_pass = LightPass::new(&gfx).await;
        let model_pass = ModelPass::new(&gfx).await;

        let camera_resource = gfx.create_camera_resource(&model_pass.layouts[1]);

        let camera = Camera::new(
            (0.0, 5.0, 10.0),
            CameraProjection::new(
                gfx.width(),
                gfx.height(),
                (45.0 as f32).to_radians(), 
                0.1,
                100.0
            ),
            (-90.0 as f32).to_radians(),
            (-20.0 as f32).to_radians());

        let camera_controller = CameraController::new(4.0, 0.4);

        let light_resource = gfx.create_light_resource(&model_pass.layouts[2]);
        let light = Light::new(
            (8.0, 2.0, 8.0),
            (1., 1., 0.9));

        let instances = Self::load_scene();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = gfx.create_instance_buffer(&instance_data);

        let depth_texture = Texture::create_depth_texture(gfx, "depth_texture");

        let obj_model = resource::load_model("cube.obj", gfx.device(), gfx.queue(), &model_pass.layouts[0]).await.unwrap();

        Scene {
            light_pass,
            model_pass,
            camera,
            camera_resource,
            camera_controller,
            light,
            light_resource,
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
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
        let position: Vec3 = (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (60.0 * dt).to_radians())
            * old_position).into();

        self.light.update(position);
        self.light_resource.update(&gfx, &self.light);
    }

    pub fn load_scene() -> Vec<Instance> {
        info!("Load scene");

        let map = include_str!("../assets/map.dat");

        let mut instances = Vec::new();

        let (mut i, mut j) = (0., 0.);

        for c in map.chars() {
            match c {
                'w' => {
                    i += TILE_SIZE;
                    let position = Vec3 {x: i - 32., y: 0.0, z: j - 32.};
                    let rotation = Quat::from_axis_angle(Vec3::Z, 0.0);

                    instances.push(Instance {
                        position, rotation
                    });
                }, '.' | '@' => {
                    i += TILE_SIZE;
                }, '\n' => {
                    j += TILE_SIZE;
                    i = 0.;
                },
                _ => ()
            }
        };

        instances
    }
}