use std::sync::Arc;

use glam::{Vec2, Vec4};
use gobs_material::vertex::VertexData;

use crate::mesh::Mesh;

const T_MIN: f32 = 0.01;
const T_MAX: f32 = 1. - T_MIN;
const PADDING: bool = true;

pub struct Shapes;

impl Shapes {
    pub fn triangle(color1: [f32; 4], color2: [f32; 4], color3: [f32; 4]) -> Arc<Mesh> {
        let mut builder = Mesh::builder("triangle");

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
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color(c[ci[i] - 1].into())
                .normal(n[ni[i] - 1].into())
                .texture(t[ti[i] - 1].into())
                .padding(PADDING)
                .build();

            builder = builder.vertex(vertex_data)
        }

        builder.build()
    }

    pub fn rect(top: f32, bottom: f32, left: f32, right: f32) -> Arc<Mesh> {
        let mut builder = Mesh::builder("rect");

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
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color([1., 1., 1., 1.].into())
                .texture(t[ti[i] - 1].into())
                .normal(n[ni[i] - 1].into())
                .padding(PADDING)
                .build();

            builder = builder.vertex(vertex_data);
        }

        builder.build()
    }

    pub fn quad() -> Arc<Mesh> {
        Self::rect(0.5, -0.5, -0.5, 0.5)
    }

    pub fn cube(cols: u32, rows: u32, index: &[u32]) -> Arc<Mesh> {
        let mut builder = Mesh::builder("cube");

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
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color(Vec4::new(1., 1., 1., 1.))
                .texture(Self::tex_map(
                    t[ti[i] - 1].into(),
                    cols,
                    rows,
                    index[(i / index.len()) % index.len()],
                ))
                .normal(n[ni[i] - 1].into())
                .padding(PADDING)
                .build();

            builder = builder.vertex(vertex_data);
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
