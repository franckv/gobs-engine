use crate::Gfx;
use crate::scene::Scene;
use crate::pipeline::{ Generator, Pipeline, PipelineBuilder };
use crate::model::{ Model, ModelVertex, Texture, Vertex };

const SHADER: &str = "../shaders/light.wgsl";

pub struct LightPass {
    pub pipeline: Pipeline,
}

impl LightPass {
    pub async fn new(gfx: &Gfx) -> Self {
        let generator = Generator::new(SHADER).await;
        let layouts = generator.bind_layouts(gfx);

        let pipeline = PipelineBuilder::new(gfx.device(), "Light pipeline")
            .shader(SHADER).await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(ModelVertex::desc())
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        LightPass {
            pipeline
        }    
    }
}

pub trait DrawLightPass<'a> {
    fn draw_light_pass(&mut self, pass: &'a LightPass, model: &'a Model, scene: &'a Scene);
}

impl <'a> DrawLightPass<'a> for wgpu::RenderPass<'a> {
    fn draw_light_pass(&mut self, pass: &'a LightPass, model: &'a Model, scene: &'a Scene) {
        self.set_pipeline(&pass.pipeline.pipeline);
        self.set_bind_group(0, &scene.camera().resource.bind_group, &[]);
        self.set_bind_group(1, &scene.light().resource.bind_group, &[]);
        for mesh in &model.meshes {
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }
}