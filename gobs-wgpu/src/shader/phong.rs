use crate::camera::CameraResource;
use crate::light::LightResource;
use crate::model::{Model, Texture};
use crate::pipeline::{Generator, Pipeline, PipelineBuilder};
use crate::render::Gfx;

const SHADER: &str = "../shaders/shader.wgsl";

pub struct PhongShader {
    pub pipeline: Pipeline,
    pub layouts: Vec<wgpu::BindGroupLayout>,
}

impl PhongShader {
    pub async fn new(gfx: &Gfx) -> Self {
        let generator = Generator::new(SHADER).await;
        let layouts = generator.bind_layouts(gfx);

        let vertex_attributes = generator.vertex_layout_attributes("VertexInput");
        let vertex_layout = generator.vertex_layout(&vertex_attributes, false);

        let instance_attributes = generator.vertex_layout_attributes("InstanceInput");
        let instance_layout = generator.vertex_layout(&instance_attributes, true);

        let pipeline = PipelineBuilder::new(gfx.device(), "Model pipeline")
            .shader(SHADER)
            .await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(vertex_layout)
            .vertex_layout(instance_layout)
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        PhongShader { pipeline, layouts }
    }
}

pub trait DrawPhong<'a> {
    fn draw_phong(
        &mut self,
        shader: &'a PhongShader,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
        instance_buffer: &'a wgpu::Buffer,
        instances: u32,
    );
}

impl<'a> DrawPhong<'a> for wgpu::RenderPass<'a> {
    fn draw_phong(
        &mut self,
        shader: &'a PhongShader,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
        instance_buffer: &'a wgpu::Buffer,
        instances: u32,
    ) {
        self.set_pipeline(&shader.pipeline.pipeline);
        self.set_bind_group(0, &camera.bind_group, &[]);
        self.set_bind_group(1, &light.bind_group, &[]);
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.set_bind_group(2, &material.bind_group, &[]);
            self.draw_indexed(0..mesh.num_elements, 0, 0..instances);
        }
    }
}
