use std::sync::Arc;

use glam::Vec2;

use gobs_core::Color;

use crate::geometry::{Mesh, VertexData};

const T_MIN: f32 = 0.01;
const T_MID: f32 = 0.5;
const T_MAX: f32 = 1. - T_MIN;

pub struct Shapes;

impl Shapes {
    pub fn triangle(
        color1: Color,
        color2: Color,
        color3: Color,
        size: f32,
        padding: bool,
    ) -> Arc<Mesh> {
        let mut builder = Mesh::builder("triangle");

        let (top, bottom, left, right) = (size / 2., -size / 2., -size / 2., size / 2.);

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

        let ni = [1, 1, 1];

        for i in 0..vi.len() {
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color(c[vi[i] - 1])
                .normal(n[ni[i] - 1].into())
                .texture(t[vi[i] - 1].into())
                .padding(padding)
                .build();

            builder = builder.vertex(vertex_data)
        }

        builder.build()
    }

    pub fn rect(
        color: Color,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
        padding: bool,
    ) -> Arc<Mesh> {
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

        for i in 0..vi.len() {
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color(color)
                .texture(t[vi[i] - 1].into())
                .normal(n[ni[i] - 1].into())
                .padding(padding)
                .build();

            builder = builder.vertex(vertex_data);
        }

        builder.build()
    }

    pub fn quad(color: Color, padding: bool) -> Arc<Mesh> {
        Self::rect(color, 0.5, -0.5, -0.5, 0.5, padding)
    }

    pub fn hexagon(colors: [Color; 7], padding: bool) -> Arc<Mesh> {
        let mut builder = Mesh::builder("hexagon");

        let width = 1.;
        let height = 3.0f32.sqrt() / 2.;

        let center = [0., 0., 0.];
        let ne = [width / 2., height, 0.];
        let e = [width, 0., 0.];
        let se = [width / 2., -height, 0.];
        let sw = [-width / 2., -height, 0.];
        let w = [-width, 0., 0.];
        let nw = [-width / 2., height, 0.];

        let v = [center, ne, e, se, sw, w, nw];

        let n = [[0., 0., 1.]];

        let t = [
            [T_MID, T_MID],
            [T_MAX, T_MAX],
            [T_MAX, T_MID],
            [T_MAX, T_MIN],
            [T_MIN, T_MIN],
            [T_MIN, T_MID],
            [T_MIN, T_MAX],
        ];

        let vi = [1, 3, 2, 1, 4, 3, 1, 5, 4, 1, 6, 5, 1, 7, 6, 1, 2, 7];

        let ni = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

        for i in 0..vi.len() {
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color(colors[vi[i] - 1])
                .texture(t[vi[i] - 1].into())
                .normal(n[ni[i] - 1].into())
                .padding(padding)
                .build();

            builder = builder.vertex(vertex_data);
        }

        builder.build()
    }

    pub fn cubemap(cols: u32, rows: u32, index: &[u32], size: f32, padding: bool) -> Arc<Mesh> {
        let mut builder = Mesh::builder("cube");

        let (top, bottom, left, right, front, back) = (
            size / 2.,
            -size / 2.,
            -size / 2.,
            size / 2.,
            size / 2.,
            -size / 2.,
        );

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
            let vertex_data = VertexData::builder()
                .position(v[vi[i] - 1].into())
                .color(Color::WHITE)
                .texture(Self::tex_map(
                    t[ti[i] - 1].into(),
                    cols,
                    rows,
                    index[(i / index.len()) % index.len()],
                ))
                .normal(n[ni[i] - 1].into())
                .padding(padding)
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
