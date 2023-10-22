use gobs_wgpu as render;

use render::{
    model::{Mesh, MeshBuilder},
    render::Gfx,
    shader_data::VertexFlag,
};
pub struct Shapes;

impl Shapes {
    pub fn triangle(gfx: &Gfx, flags: VertexFlag) -> Mesh {
        let mut builder = MeshBuilder::new("triangle", flags);

        let (top, bottom, left, right) = (0.5, -0.5, -0.5, 0.5);

        let v1 = [left, bottom, 0.];
        let v2 = [right, bottom, 0.];
        let v3 = [(left + right) / 2., top, 0.];

        let n = [0., 0., 1.];

        let t1 = [0., 1.];
        let t2 = [1., 1.];
        let t3 = [0.5, 0.];

        match flags {
            VertexFlag::POSITION => {
                builder = builder
                    .add_vertex_P(v1.into())
                    .add_vertex_P(v2.into())
                    .add_vertex_P(v3.into())
            }
            VertexFlag::PTN => {
                builder = builder
                    .add_vertex_PTN(v1.into(), t1.into(), n.into())
                    .add_vertex_PTN(v2.into(), t2.into(), n.into())
                    .add_vertex_PTN(v3.into(), t3.into(), n.into())
            }
            _ => todo!("triangle"),
        }
        builder.build(gfx)
    }
}
