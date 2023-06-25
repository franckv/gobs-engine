mod material;
mod mesh;
mod texture;

pub use material::Material;
pub use mesh::Mesh;
pub use texture::Texture;

use std::ops::Range;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3]
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>
}

pub trait DrawModel<'a> {
    fn draw_model(&mut self, model: &'a Model);
    fn draw_model_instanced(&mut self, model: &'a Model, instances: Range<u32>);
}

impl <'a> DrawModel<'a> for wgpu::RenderPass<'a> {
    fn draw_model(&mut self, model: &'a Model) {
        self.draw_model_instanced(model, 0..1);
    }

    fn draw_model_instanced(
        &mut self, 
        model: &'a Model, 
        instances: Range<u32>) {
            for mesh in &model.meshes {
                let material = &model.materials[mesh.material];
                self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                self.set_bind_group(0, &material.bind_group, &[]);
                self.draw_indexed(0..mesh.num_elements, 0, instances.clone());
            }
        }
}

pub trait DrawLight<'a> {
    fn draw_light_model(&mut self, model: &'a Model);
    fn draw_light_model_instanced(&mut self, model: &'a Model, instances: Range<u32>);
}

impl <'a> DrawLight<'a> for wgpu::RenderPass<'a> {
    fn draw_light_model(&mut self, model: &'a Model) {
        self.draw_light_model_instanced(model, 0..1);
    }

    fn draw_light_model_instanced(
        &mut self, 
        model: &'a Model, 
        instances: Range<u32>) {
            for mesh in &model.meshes {
                self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                self.draw_indexed(0..mesh.num_elements, 0, instances.clone());
            }
        }
}