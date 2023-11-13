use std::sync::Arc;

use uuid::Uuid;

use crate::{
    context::Gfx,
    model::{InstanceFlag, Texture, VertexFlag},
    pipeline::{Generator, Pipeline, PipelineBuilder, PipelineFlag},
};

pub enum ShaderBindGroup {
    Camera,
    Light,
    Material,
}

pub type ShaderId = Uuid;

pub struct Shader {
    pub id: ShaderId,
    pub name: String,
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
            .color_format(gfx.format())
            .depth_format(Texture::DEPTH_FORMAT)
            .build();

        Arc::new(Shader {
            id: Uuid::new_v4(),
            name: name.to_string(),
            pipeline,
            layouts,
            vertex_flags,
            instance_flags,
            pipeline_flags,
        })
    }

    pub fn layout(&self, id: ShaderBindGroup) -> &wgpu::BindGroupLayout {
        match id {
            ShaderBindGroup::Camera => &self.layouts[0],
            ShaderBindGroup::Light => &self.layouts[1],
            ShaderBindGroup::Material => &self.layouts[2],
        }
    }
}
