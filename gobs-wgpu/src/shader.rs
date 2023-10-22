mod phong;
mod solid;

pub use phong::PhongShader;
pub use solid::SolidShader;

use crate::{
    model::{CameraResource, LightResource, Model},
    shader_data::InstanceFlag,
};

#[derive(Copy, Clone)]
pub enum ShaderType {
    Phong,
    Solid,
}

impl ShaderType {
    pub fn instance_flags(&self) -> InstanceFlag {
        match self {
            ShaderType::Phong => InstanceFlag::MN,
            ShaderType::Solid => InstanceFlag::MODEL,
        }
    }
}

pub enum Shader {
    Phong(PhongShader),
    Solid(SolidShader),
}

impl Shader {
    pub fn layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        match self {
            Shader::Phong(shader) => &shader.layouts,
            Shader::Solid(shader) => &shader.layouts,
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
    ) {
    }

    fn draw_instanced(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        model: &'a Model,
        camera: &'a CameraResource,
        light: &'a LightResource,
        instance_buffer: &'a wgpu::Buffer,
        instances: usize,
    ) {
    }
}
