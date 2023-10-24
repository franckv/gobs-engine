mod phong;
mod solid;

pub use phong::PhongShader;
pub use solid::SolidShader;

use crate::{
    model::{CameraResource, LightResource, Model},
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

impl ShaderType {
    pub fn instance_flags(&self) -> InstanceFlag {
        match self {
            ShaderType::Phong => InstanceFlag::MODEL | InstanceFlag::NORMAL,
            ShaderType::Solid => InstanceFlag::MODEL,
        }
    }

    pub fn vertex_flags(&self) -> VertexFlag {
        match self {
            ShaderType::Phong => VertexFlag::POSITION | VertexFlag::TEXTURE | VertexFlag::NORMAL,
            ShaderType::Solid => VertexFlag::POSITION,
        }
    }
}

pub enum Shader {
    Phong(PhongShader),
    Solid(SolidShader),
}

impl Shader {
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
