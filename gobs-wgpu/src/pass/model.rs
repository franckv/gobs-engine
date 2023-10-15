use wgpu::BindGroupLayout;

use crate::Gfx;
use crate::InstanceRaw;
use crate::scene::Scene;
use crate::pipeline::{ Generator, Pipeline, PipelineBuilder };
use crate::model::{ Model, ModelVertex, Texture, Vertex };
use crate::camera::CameraResource;
use crate::light::LightResource;

const SHADER: &str = "../shaders/shader.wgsl";

pub struct ModelPass {
    pub pipeline: Pipeline,
    pub layouts: Vec<BindGroupLayout>
}

impl ModelPass {
    pub async fn new(gfx: &Gfx) -> Self {
        let generator = Generator::new(SHADER).await;
        let layouts = generator.bind_layouts(gfx);

        let pipeline = PipelineBuilder::new(gfx.device(), "Light pipeline")
            .shader(SHADER).await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(ModelVertex::desc())
            .vertex_layout(InstanceRaw::desc())
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        ModelPass {
            pipeline,
            layouts
        }    
    }
}

pub trait DrawModelPass<'a> {
    fn draw_model_pass(&mut self, pass: &'a ModelPass, model: &'a Model, camera: &'a CameraResource, light: &'a LightResource, scene: &'a Scene);
}

impl <'a> DrawModelPass<'a> for wgpu::RenderPass<'a> {
    fn draw_model_pass(&mut self, pass: &'a ModelPass, model: &'a Model, camera: &'a CameraResource, light: &'a LightResource, scene: &'a Scene) {
        self.set_pipeline(&pass.pipeline.pipeline);
        self.set_bind_group(1, &camera.bind_group, &[]);
        self.set_bind_group(2, &light.bind_group, &[]);
        self.set_vertex_buffer(1, scene.instance_buffer().slice(..));
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.set_bind_group(0, &material.bind_group, &[]);
            self.draw_indexed(0..mesh.num_elements, 0, 0..scene.instances().len() as _);
        }
    }
}