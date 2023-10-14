use glam::{ Quat, Vec3 };
use wgpu::util::DeviceExt;

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

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN: f32 = 3.0;

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
        let layouts = generator.bind_layouts(gfx.device());

        let generator_light = Generator::new("../shaders/light.wgsl").await;
        let layouts_light = generator_light.bind_layouts(gfx.device());

        let camera_resource = gfx.create_camera_resource(&layouts[1]);

        let camera = Camera::new(
            camera_resource,
            (0.0, 5.0, 10.0),
            CameraProjection::new(
                gfx.config().width,
                gfx.config().height,
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
            (2.0, 2.0, 2.0),
            (1., 1., 0.9));

        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = Vec3 {x, y: 0.0, z };
                let rotation = if position == Vec3::ZERO {
                    Quat::from_axis_angle(Vec3::Z, 0.0)
                } else {
                    Quat::from_axis_angle(position.normalize(), (45.0 as f32).to_radians())
                };

                Instance {
                    position, rotation
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = gfx.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let depth_texture = Texture::create_depth_texture(gfx.device(), gfx.config(), "depth_texture");

        let obj_model = resource::load_model("cube.obj", gfx.device(), gfx.queue(), &layouts[0]).await.unwrap();

        let render_pipeline = PipelineBuilder::new(gfx.device(), "Render pipeline")
            .shader("../shaders/shader.wgsl").await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(ModelVertex::desc())
            .vertex_layout(InstanceRaw::desc())
            .color_format(gfx.config().format)
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        let light_render_pipeline = PipelineBuilder::new(gfx.device(), "Light pipeline")
            .shader("../shaders/light.wgsl").await
            .bind_layout(layouts_light.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(ModelVertex::desc())
            .color_format(gfx.config().format)
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

    pub fn resize(&mut self, gfx: &Gfx, new_size: winit::dpi::PhysicalSize<u32>) {
        self.depth_texture = Texture::create_depth_texture(gfx.device(), gfx.config(), "depth_texture");
        self.camera.projection.resize(new_size.width, new_size.height);
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera.update_view_proj();

        let old_position: Vec3 = self.light.position;
        let position: Vec3 = (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), (60.0 * dt).to_radians())
            * old_position).into();

        self.light.update(position);
    }
}