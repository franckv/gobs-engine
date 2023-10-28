mod phong;
mod solid;

use std::sync::Arc;

pub use phong::PhongShader;
pub use solid::SolidShader;

use crate::{
    model::{CameraResource, LightResource, Model},
    render::Gfx,
    shader_data::{InstanceFlag, VertexFlag},
};

pub enum ShaderBindGroup {
    Camera,
    Light,
    Material,
}

#[derive(Copy, Clone)]
pub enum ShaderType {
    Phong,
    Solid,
}

pub enum Shader {
    Phong(PhongShader),
    Solid(SolidShader),
}

impl Shader {
    pub async fn new(gfx: &Gfx, ty: ShaderType) -> Arc<Self> {
        match ty {
            ShaderType::Phong => PhongShader::new(gfx).await,
            ShaderType::Solid => SolidShader::new(gfx).await,
        }
    }

    pub fn instance_flags(&self) -> InstanceFlag {
        match self {
            Shader::Phong(_) => PhongShader::instance_flags(),
            Shader::Solid(_) => SolidShader::instance_flags(),
        }
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        match self {
            Shader::Phong(_) => PhongShader::vertex_flags(),
            Shader::Solid(_) => SolidShader::vertex_flags(),
        }
    }

    pub fn ty(&self) -> ShaderType {
        match self {
            Shader::Phong(_) => ShaderType::Phong,
            Shader::Solid(_) => ShaderType::Solid,
        }
    }

    pub fn layout(&self, id: ShaderBindGroup) -> &wgpu::BindGroupLayout {
        match self {
            Shader::Phong(shader) => shader.layout(id),
            Shader::Solid(shader) => shader.layout(id),
        }
    }
}

#[allow(unused_variables)]
pub trait ShaderDraw<'a, 'b>
where
    'a: 'b,
{
    fn instance_flags() -> InstanceFlag;

    fn vertex_flags() -> VertexFlag;

    fn draw(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
    );

    fn draw_instanced(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
        instance_buffer: &'a wgpu::Buffer,
        instances: usize,
    );

    fn layout(&self, id: ShaderBindGroup) -> &wgpu::BindGroupLayout;
}
