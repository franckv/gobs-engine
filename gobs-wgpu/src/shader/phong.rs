use crate::model::CameraResource;
use crate::model::LightResource;
use crate::model::{Model, Texture};
use crate::pipeline::{Generator, Pipeline, PipelineBuilder};
use crate::render::Gfx;

use crate::shader::Shader;
use crate::shader::ShaderDraw;

const SHADER: &str = "../shaders/shader.wgsl";

pub struct PhongShader {
    pub pipeline: Pipeline,
    pub layouts: Vec<wgpu::BindGroupLayout>,
}

impl PhongShader {
    pub async fn new(gfx: &Gfx) -> Shader {
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

        Shader::Phong(PhongShader { pipeline, layouts })
    }
}

impl<'a, 'b> ShaderDraw<'a, 'b> for PhongShader
where
    'a: 'b,
{
    fn draw_instanced(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
        instance_buffer: &'a wgpu::Buffer,
        instances: usize,
    ) {
        render_pass.set_pipeline(&self.pipeline.pipeline);
        render_pass.set_bind_group(0, &camera.bind_group, &[]);
        render_pass.set_bind_group(1, &light.bind_group, &[]);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(2, &material.bind_group, &[]);
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..instances as _);
        }
    }
}
