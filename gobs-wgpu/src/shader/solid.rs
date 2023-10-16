use crate::camera::CameraResource;
use crate::light::LightResource;
use crate::model::{ Model, ModelVertex, Texture, Vertex };
use crate::pipeline::{ Generator, Pipeline, PipelineBuilder };
use crate::render::Gfx;

const SHADER: &str = "../shaders/light.wgsl";

pub struct SolidShader {
    pub pipeline: Pipeline,
}

impl SolidShader {
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

        SolidShader {
            pipeline
        }    
    }
}

pub trait DrawSolid<'a> {
    fn draw_solid(&mut self, shader: &'a SolidShader, model: &'a Model, camera: &'a CameraResource, light: &'a LightResource);
}

impl <'a> DrawSolid<'a> for wgpu::RenderPass<'a> {
    fn draw_solid(&mut self, shader: &'a SolidShader, model: &'a Model, camera: &'a CameraResource, light: &'a LightResource) {
        self.set_pipeline(&shader.pipeline.pipeline);
        self.set_bind_group(0, &camera.bind_group, &[]);
        self.set_bind_group(1, &light.bind_group, &[]);
        for mesh in &model.meshes {
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }
}