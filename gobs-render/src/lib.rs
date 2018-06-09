#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;
extern crate cgmath;
extern crate time;
extern crate uuid;
#[macro_use] extern crate log;

extern crate gobs_scene as scene;

pub mod cache;
pub mod context;
pub mod display;
pub mod render;
pub mod pipeline;

pub use render::{Batch, Command, Renderer};

pub use scene::model::{Instance, Vertex};

#[derive(Copy, Clone)]
pub struct RenderVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_uv: [f32; 2],
}

impl From<Vertex> for RenderVertex {
    fn from(v: Vertex) -> Self {
        RenderVertex {
            position: v.position,
            normal: v.normal,
            tex_uv: v.tex_uv
        }
    }
}

#[derive(Copy, Clone)]
pub struct RenderInstance {
    pub transform: [[f32; 4]; 4],
    pub normal_transform: [[f32; 3]; 3],
    pub color: [f32; 4],
    pub region: [f32; 4],
}

impl From<Instance> for RenderInstance {
    fn from(i: Instance) -> Self {
        RenderInstance {
            transform: i.transform,
            normal_transform: i.normal_transform,
            color: i.color,
            region: i.region
        }
    }
}

impl_vertex!(RenderVertex, position, normal, tex_uv);
impl_vertex!(RenderInstance, transform, normal_transform, color, region);
