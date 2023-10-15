use glam::{ Quat, Vec3 };

use crate::Camera;
use crate::CameraController;
use crate::camera::CameraProjection;
use crate::Gfx;
use crate::Instance;
use crate::InstanceRaw;
use crate::Light;
use crate::pipeline::{ Generator, Pipeline, PipelineBuilder };
use crate::resource;
use crate::model::{ Model, ModelVertex, Texture, Vertex };

const TILE_SIZE: f32 = 2.;

pub struct Scene {
    render_pipeline: Pipeline,
    light_render_pipeline: Pipeline,
    camera: Camera,
    pub camera_controller: CameraController,
    light: Light,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    obj_model: Model,
}

impl Scene {
    pub fn render_pipeline(&self) -> &Pipeline {
        &self.render_pipeline
    }

    pub fn light_render_pipeline(&self) -> &Pipeline {
        &self.light_render_pipeline
    }

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
        let generator = Generator::new("../shaders/shader.wgsl").await;
        let layouts = generator.bind_layouts(gfx);

        let generator_light = Generator::new("../shaders/light.wgsl").await;
        let layouts_light = generator_light.bind_layouts(gfx);

        let camera_resource = gfx.create_camera_resource(&layouts[1]);

        let camera = Camera::new(
            camera_resource,
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

        let light_resource = gfx.create_light_resource(&layouts[2]);
        let light = Light::new(
            light_resource,
            (8.0, 2.0, 8.0),
            (1., 1., 0.9));

        let instances = Self::load_scene();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = gfx.create_instance_buffer(&instance_data);

        let depth_texture = Texture::create_depth_texture(gfx, "depth_texture");

        let obj_model = resource::load_model("cube.obj", gfx.device(), gfx.queue(), &layouts[0]).await.unwrap();

        let render_pipeline = PipelineBuilder::new(gfx.device(), "Render pipeline")
            .shader("../shaders/shader.wgsl").await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(ModelVertex::desc())
            .vertex_layout(InstanceRaw::desc())
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        let light_render_pipeline = PipelineBuilder::new(gfx.device(), "Light pipeline")
            .shader("../shaders/light.wgsl").await
            .bind_layout(layouts_light.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(ModelVertex::desc())
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        Scene {
            render_pipeline,
            light_render_pipeline,
            camera,
            camera_controller,
            light,
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
        self.camera.update_view_proj();

        let old_position: Vec3 = self.light.position;
        let position: Vec3 = (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (60.0 * dt).to_radians())
            * old_position).into();

        self.light.update(position);

        gfx.queue().write_buffer(&self.camera.resource.buffer, 0, bytemuck::cast_slice(&[self.camera.resource.uniform]));
        gfx.queue().write_buffer(&self.light.resource.buffer, 0, bytemuck::cast_slice(&[self.light.resource.uniform]));
    }

    pub fn load_scene() -> Vec<Instance> {

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