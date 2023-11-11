use std::sync::Arc;

use crate::{
    model::{InstanceFlag, Model, Texture, VertexFlag},
    pipeline::{Generator, Pipeline, PipelineBuilder, PipelineFlag},
    render::Gfx,
    resources::{CameraResource, LightResource},
};

pub enum ShaderBindGroup {
    Camera,
    Light,
    Material,
}

pub struct Shader {
    pub pipeline: Pipeline,
    layouts: Vec<wgpu::BindGroupLayout>,
    pub vertex_flags: VertexFlag,
    pub instance_flags: InstanceFlag,
    pub pipeline_flags: PipelineFlag,
}

impl Shader {
    pub async fn new(
        gfx: &Gfx,
        name: &str,
        file: &str,
        vertex_flags: VertexFlag,
        instance_flags: InstanceFlag,
        pipeline_flags: PipelineFlag,
    ) -> Arc<Self> {
        let generator = Generator::new(file).await;
        let layouts = generator.bind_layouts(gfx);

        let vertex_attributes = generator.vertex_layout_attributes("VertexInput");
        let vertex_layout =
            generator.vertex_layout(&vertex_attributes, false, instance_flags, vertex_flags);

        let instance_attributes = generator.vertex_layout_attributes("InstanceInput");
        let instance_layout =
            generator.vertex_layout(&instance_attributes, true, instance_flags, vertex_flags);

        let pipeline = PipelineBuilder::new(gfx.device(), name, pipeline_flags)
            .shader(file)
            .await
            .bind_layout(layouts.iter().collect::<Vec<_>>().as_slice())
            .vertex_layout(vertex_layout)
            .vertex_layout(instance_layout)
            .color_format(gfx.format().clone())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        Arc::new(Shader {
            pipeline,
            layouts,
            vertex_flags,
            instance_flags,
            pipeline_flags,
        })
    }

    pub fn draw<'a, 'b>(
        &'a self,
        _render_pass: &mut wgpu::RenderPass<'b>,
        _model: &'a Model,
        _camera: &'a CameraResource,
        _light: &'a LightResource,
    ) where
        'a: 'b,
    {
        todo!()
    }

    pub fn draw_instanced<'a, 'b>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
        instance_buffer: &'a wgpu::Buffer,
        instances: usize,
    ) where
        'a: 'b,
    {
        render_pass.set_pipeline(&self.pipeline.pipeline);
        render_pass.set_bind_group(0, &camera.bind_group, &[]);
        render_pass.set_bind_group(1, &light.bind_group, &[]);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        for mesh in &model.mesh_data {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            if let Some(bind_group) = &mesh.bind_group {
                render_pass.set_bind_group(2, &bind_group, &[]);
            }
            render_pass.draw_indexed(0..mesh.num_elements as _, 0, 0..instances as _);
        }
    }

    pub fn layout(&self, id: ShaderBindGroup) -> &wgpu::BindGroupLayout {
        match id {
            ShaderBindGroup::Camera => &self.layouts[0],
            ShaderBindGroup::Light => &self.layouts[1],
            ShaderBindGroup::Material => &self.layouts[2],
        }
    }
}