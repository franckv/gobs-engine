use std::sync::Arc;

use glam::{Vec2, Vec4};
use gobs_wgpu as render;

use render::model::{Mesh, MeshBuilder};

const T_MIN: f32 = 0.01;
const T_MAX: f32 = 1. - T_MIN;

pub struct Shapes;

impl Shapes {
    pub fn triangle(color1: [f32; 4], color2: [f32; 4], color3: [f32; 4]) -> Arc<Mesh> {
        let mut builder = MeshBuilder::new("triangle");

        let (top, bottom, left, right) = (0.5, -0.5, -0.5, 0.5);

        let v = [
            [left, bottom, 0.],
            [right, bottom, 0.],
            [(left + right) / 2., top, 0.],
        ];

        let c = [color1, color2, color3];

        let n = [[0., 0., 1.]];

        let t = [
            [T_MIN, T_MAX],
            [T_MAX, T_MAX],
            [(T_MIN + T_MAX) / 2., T_MIN],
        ];

        let vi = [1, 2, 3];

        let ci = [1, 2, 3];

        let ni = [1, 1, 1];

        let ti = [1, 2, 3];

        for i in 0..vi.len() {
            builder = builder.add_vertex(
                v[vi[i] - 1].into(),
                c[ci[i] - 1].into(),
                t[ti[i] - 1].into(),
                n[ni[i] - 1].into(),
                t[ti[i] - 1].into(),
            )
        }

        builder.build()
    }

    pub fn quad() -> Arc<Mesh> {
        let mut builder = MeshBuilder::new("quad");

        let (top, bottom, left, right) = (0.5, -0.5, -0.5, 0.5);

        let v = [
            [left, top, 0.],
            [right, top, 0.],
            [left, bottom, 0.],
            [right, bottom, 0.],
        ];

        let n = [[0., 0., 1.]];

        let t = [
            [T_MIN, T_MIN],
            [T_MAX, T_MIN],
            [T_MIN, T_MAX],
            [T_MAX, T_MAX],
        ];

        let vi = [1, 3, 4, 4, 2, 1];

        let ni = [1, 1, 1, 1, 1, 1];

        let ti = [1, 3, 4, 4, 2, 1];

        for i in 0..vi.len() {
            builder = builder.add_vertex(
                v[vi[i] - 1].into(),
                [1., 1., 1., 1.].into(),
                t[ti[i] - 1].into(),
                n[ni[i] - 1].into(),
                t[ti[i] - 1].into(),
            )
        }

        builder.build()
    }

    pub fn cube(cols: u32, rows: u32, index: &[u32]) -> Arc<Mesh> {
        let mut builder = MeshBuilder::new("cube");

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

        let t = [
            [T_MIN, T_MIN],
            [T_MAX, T_MIN],
            [T_MIN, T_MAX],
            [T_MAX, T_MAX],
        ];

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
            builder = builder.add_vertex(
                v[vi[i] - 1].into(),
                Vec4::new(1., 1., 1., 1.),
                Self::tex_map(
                    t[ti[i] - 1].into(),
                    cols,
                    rows,
                    index[(i / index.len()) % index.len()],
                ),
                n[ni[i] - 1].into(),
                t[ti[i] - 1].into(),
            )
        }

        builder.build()
    }

    fn tex_map(tex_coords: Vec2, cols: u32, rows: u32, index: u32) -> Vec2 {
        let col = ((index - 1) % cols) as f32;
        let row = ((index - 1) / cols) as f32;

        let u = (col + tex_coords.x) / cols as f32;
        let v = (row + tex_coords.y) / rows as f32;

        Vec2::new(u, v)
    }
}
