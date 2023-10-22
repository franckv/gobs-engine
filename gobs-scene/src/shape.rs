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

        let v = [
            [left, bottom, 0.],
            [right, bottom, 0.],
            [(left + right) / 2., top, 0.],
        ];

        let n = [[0., 0., 1.]];

        let t = [[0., 1.], [1., 1.], [0.5, 0.]];

        let vi = [1, 2, 3];

        let ni = [1, 1, 1];

        let ti = [1, 2, 3];

        match flags {
            VertexFlag::POSITION => {
                for i in 0..vi.len() {
                    builder = builder.add_vertex_P(v[vi[i] - 1].into())
                }
            }
            VertexFlag::PTN => {
                for i in 0..vi.len() {
                    builder = builder.add_vertex_PTN(
                        v[vi[i] - 1].into(),
                        t[ti[i] - 1].into(),
                        n[ni[i] - 1].into(),
                    )
                }
            }
            _ => todo!("triangle"),
        }
        builder.build(gfx)
    }

    pub fn quad(gfx: &Gfx, flags: VertexFlag) -> Mesh {
        let mut builder = MeshBuilder::new("quad", flags);

        let (top, bottom, left, right) = (0.5, -0.5, -0.5, 0.5);

        let v = [
            [left, top, 0.],
            [right, top, 0.],
            [left, bottom, 0.],
            [right, bottom, 0.],
        ];

        let n = [[0., 0., 1.]];

        let t = [[0., 0.], [1., 0.], [0., 1.], [1., 1.]];

        let vi = [1, 3, 4, 4, 2, 1];

        let ni = [1, 1, 1, 1, 1, 1];

        let ti = [1, 3, 4, 4, 2, 1];

        match flags {
            VertexFlag::POSITION => {
                for i in 0..vi.len() {
                    builder = builder.add_vertex_P(v[vi[i] - 1].into())
                }
            }
            VertexFlag::PTN => {
                for i in 0..vi.len() {
                    builder = builder.add_vertex_PTN(
                        v[vi[i] - 1].into(),
                        t[ti[i] - 1].into(),
                        n[ni[i] - 1].into(),
                    )
                }
            }
            _ => todo!("quad"),
        }
        builder.build(gfx)
    }

    pub fn cube(gfx: &Gfx, flags: VertexFlag) -> Mesh {
        let mut builder = MeshBuilder::new("cube", flags);

        let (top, bottom, left, right, front, back) = (0.5, -0.5, -0.5, 0.5, 0.5, -0.5);

        let v = [
            [left, top, front],
            [right, top, front],
            [left, bottom, front],
            [right, bottom, front],
            [left, top, back],
            [right, top, back],
            [left, bottom, back],
            [right, bottom, back],
        ];

        let n = [
            [0., 0., 1.],
            [0., 0., -1.],
            [-1., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., -1., 0.],
        ];

        let t = [[0., 0.], [1., 0.], [0., 1.], [1., 1.]];

        let vi = [
            3, 4, 2, 3, 2, 1, 8, 7, 5, 8, 5, 6, 7, 3, 1, 7, 1, 5, 4, 8, 6, 4, 6, 2, 1, 2, 6, 1, 6,
            5, 7, 8, 4, 7, 4, 3,
        ];

        let ni = [
            1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5,
            5, 6, 6, 6, 6, 6, 6,
        ];

        let ti = [
            3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2,
            1, 3, 4, 2, 3, 2, 1,
        ];

        match flags {
            VertexFlag::POSITION => {
                for i in 0..vi.len() {
                    builder = builder.add_vertex_P(v[vi[i] - 1].into())
                }
            }
            VertexFlag::PTN => {
                for i in 0..vi.len() {
                    builder = builder.add_vertex_PTN(
                        v[vi[i] - 1].into(),
                        t[ti[i] - 1].into(),
                        n[ni[i] - 1].into(),
                    )
                }
            }
            _ => todo!("cube"),
        }
        builder.build(gfx)
    }
}
