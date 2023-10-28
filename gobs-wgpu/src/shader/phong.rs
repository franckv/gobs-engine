use crate::model::CameraResource;
use crate::model::LightResource;
use crate::model::{Model, Texture};
use crate::pipeline::{Generator, Pipeline, PipelineBuilder};
use crate::render::Gfx;

use crate::shader::Shader;
use crate::shader::ShaderBindGroup;
use crate::shader::ShaderDraw;
use crate::shader::ShaderType;
use crate::shader_data::InstanceFlag;
use crate::shader_data::VertexFlag;

const SHADER: &str = "phong.wgsl";

pub struct PhongShader {
    pub ty: ShaderType,
    pub pipeline: Pipeline,
    layouts: Vec<wgpu::BindGroupLayout>,
}

impl PhongShader {
    pub fn instance_flags() -> InstanceFlag {
        InstanceFlag::MODEL | InstanceFlag::NORMAL
    }

    pub fn vertex_flags() -> VertexFlag {
        VertexFlag::POSITION | VertexFlag::TEXTURE | VertexFlag::NORMAL
    }

    pub async fn new(gfx: &Gfx) -> Shader {
        let generator = Generator::new(SHADER).await;
        let layouts = generator.bind_layouts(gfx);
        let instance_flags = Self::instance_flags();
        let vertex_flags = Self::vertex_flags();

        let vertex_attributes = generator.vertex_layout_attributes("VertexInput");
        let vertex_layout =
            generator.vertex_layout(&vertex_attributes, false, instance_flags, vertex_flags);

        let instance_attributes = generator.vertex_layout_attributes("InstanceInput");
        let instance_layout =
            generator.vertex_layout(&instance_attributes, true, instance_flags, vertex_flags);

        let pipeline = PipelineBuilder::new(gfx.device(), "Model pipeline")
            .shader(SHADER)
            .await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(vertex_layout)
            .vertex_layout(instance_layout)
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        Shader::Phong(PhongShader {
            ty: ShaderType::Phong,
            pipeline,
            layouts,
        })
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
        for (mesh, material) in &model.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            if let Some(material) = material {
                render_pass.set_bind_group(2, &material.bind_group, &[]);
            }
            render_pass.draw_indexed(0..mesh.num_elements as _, 0, 0..instances as _);
        }
    }

    fn draw(
        &'a self,
        _render_pass: &mut wgpu::RenderPass<'b>,
        _model: &'a Model,
        _camera: &'a CameraResource,
        _light: &'a LightResource,
    ) {
        todo!()
    }

    fn layout(&self, id: ShaderBindGroup) -> &wgpu::BindGroupLayout {
        match id {
            ShaderBindGroup::Camera => &self.layouts[0],
            ShaderBindGroup::Light => &self.layouts[1],
            ShaderBindGroup::Material => &self.layouts[2],
        }
    }
}
