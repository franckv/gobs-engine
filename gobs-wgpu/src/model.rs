mod material;
mod mesh;
mod texture;

pub use material::Material;
pub use mesh::{Mesh, ModelVertex, Vertex};
pub use texture::Texture;

use std::ops::Range;

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