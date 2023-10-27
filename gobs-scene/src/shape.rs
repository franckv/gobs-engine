use glam::Vec2;
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

        for i in 0..vi.len() {
            builder = builder.add_vertex_PTNI(
                v[vi[i] - 1].into(),
                t[ti[i] - 1].into(),
                n[ni[i] - 1].into(),
                1.0,
            )
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

        for i in 0..vi.len() {
            builder = builder.add_vertex_PTN(
                v[vi[i] - 1].into(),
                t[ti[i] - 1].into(),
                n[ni[i] - 1].into(),
            )
        }

        builder.build(gfx)
    }

    pub fn cube(gfx: &Gfx, flags: VertexFlag, cols: u32, rows: u32, index: &[u32]) -> Mesh {
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
            3, 4, 2, 3, 2, 1, // F
            8, 7, 5, 8, 5, 6, // B
            7, 3, 1, 7, 1, 5, // L
            4, 8, 6, 4, 6, 2, // R
            1, 2, 6, 1, 6, 5, // U
            7, 8, 4, 7, 4, 3, // D
        ];

        let ni = [
            1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5,
            5, 6, 6, 6, 6, 6, 6,
        ];

        let ti = [
            3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2, 1, 3, 4, 2, 3, 2,
            1, 3, 4, 2, 3, 2, 1,
        ];

        for i in 0..vi.len() {
            builder = builder.add_vertex_PTN(
                v[vi[i] - 1].into(),
                //t[ti[i] - 1].into(),
                Self::tex_map(
                    t[ti[i] - 1].into(),
                    cols,
                    rows,
                    index[(i / index.len()) % index.len()],
                ),
                n[ni[i] - 1].into(),
            )
        }

        builder.build(gfx)
    }

    fn tex_map(tex_coords: Vec2, cols: u32, rows: u32, index: u32) -> Vec2 {
        let col = ((index - 1) % cols) as f32;
        let row = ((index - 1) / cols) as f32;

        let u = (col + tex_coords.x) / cols as f32;
        let v = (row + tex_coords.y) / rows as f32;

        Vec2::new(u, v)
    }

    pub fn cube_tiled(
        gfx: &Gfx,
        flags: VertexFlag,
        cols: usize,
        rows: usize,
        f: usize,
        b: usize,
        l: usize,
        r: usize,
        u: usize,
        d: usize,
    ) -> Mesh {
        let mut builder = MeshBuilder::new("cube", flags);

        let (top, bottom, left, right, front, back) = (0.5, -0.5, -0.5, 0.5, 0.5, -0.5);

        let pos = [
            [(f - 1) % cols, (f - 1) / cols], // F
            [(b - 1) % cols, (b - 1) / cols], // B
            [(l - 1) % cols, (l - 1) / cols], // L
            [(r - 1) % cols, (r - 1) / cols], // R
            [(u - 1) % cols, (u - 1) / cols], // U
            [(d - 1) % cols, (d - 1) / cols], // D
        ];

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
            3, 4, 2, 3, 2, 1, // F
            8, 7, 5, 8, 5, 6, // B
            7, 3, 1, 7, 1, 5, // L
            4, 8, 6, 4, 6, 2, // R
            1, 2, 6, 1, 6, 5, // U
            7, 8, 4, 7, 4, 3, // D
        ];

        let ni = [
            1, 1, 1, 1, 1, 1, // F
            2, 2, 2, 2, 2, 2, // B
            3, 3, 3, 3, 3, 3, // L
            4, 4, 4, 4, 4, 4, // R
            5, 5, 5, 5, 5, 5, // U
            6, 6, 6, 6, 6, 6, // D
        ];

        let ti = [
            3, 4, 2, 3, 2, 1, // F
            3, 4, 2, 3, 2, 1, // B
            3, 4, 2, 3, 2, 1, // L
            3, 4, 2, 3, 2, 1, // R
            3, 4, 2, 3, 2, 1, // U
            3, 4, 2, 3, 2, 1, // D
        ];

        for i in 0..vi.len() {
            let tex = t[ti[i] - 1];
            let tex_mapped = [
                (tex[0] + pos[i / 6][0] as f32) / (cols as f32),
                (tex[1] + pos[i / 6][1] as f32) / (rows as f32),
            ];
            builder = builder.add_vertex_PTNI(
                v[vi[i] - 1].into(),
                tex_mapped.into(),
                n[ni[i] - 1].into(),
                1.0,
            )
        }

        builder.build(gfx)
    }
}
