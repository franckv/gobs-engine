use std::sync::Arc;

use glam::Vec2;

use gobs_core::Color;
use gobs_render_hal::VertexData;

use crate::{
    MeshBuilder,
    resources::{BoundingBox, MeshGeometry},
};

const T_MIN: f32 = 0.01;
const T_MID: f32 = 0.5;
const T_MAX: f32 = 1. - T_MIN;

const QUAD_UV: [[f32; 2]; 4] = [
    [T_MIN, T_MAX],
    [T_MIN, T_MIN],
    [T_MAX, T_MAX],
    [T_MAX, T_MIN],
];

pub struct Shapes;

impl Shapes {
    pub fn triangle(colors: &[Color], size: f32) -> Arc<MeshGeometry> {
        ShapeBuilder::new("triangle")
            .colors(colors)
            .add_triangle(size, size)
            .build()
    }

    pub fn rect(
        colors: &[Color],
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
    ) -> Arc<MeshGeometry> {
        ShapeBuilder::new("rect")
            .colors(colors)
            .add_quad([
                [left, top, 0.],
                [right, top, 0.],
                [left, bottom, 0.],
                [right, bottom, 0.],
            ])
            .build()
    }

    pub fn square(colors: &[Color]) -> Arc<MeshGeometry> {
        Self::rect(colors, 0.5, -0.5, -0.5, 0.5)
    }

    pub fn hexagon(colors: &[Color]) -> Arc<MeshGeometry> {
        let width = 1.;
        let height = 3.0f32.sqrt() / 2.;

        ShapeBuilder::new("hexagon")
            .colors(colors)
            .add_hex(width, height)
            .build()
    }

    pub fn cube(colors: &[Color], size: f32) -> Arc<MeshGeometry> {
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

        ShapeBuilder::new("cube")
            .colors(colors)
            .add_quad([v[2], v[0], v[3], v[1]])
            .add_quad([v[7], v[5], v[6], v[4]])
            .add_quad([v[6], v[4], v[2], v[0]])
            .add_quad([v[3], v[1], v[7], v[5]])
            .add_quad([v[0], v[4], v[1], v[5]])
            .add_quad([v[6], v[2], v[7], v[3]])
            .build()
    }

    pub fn cubemap(cols: u32, rows: u32, index: &[u32], size: f32) -> Arc<MeshGeometry> {
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

        let uv = |i: u32| {
            QUAD_UV.map(|c| {
                Self::tex_map(c.into(), cols, rows, index[(i as usize) % index.len()]).into()
            })
        };

        ShapeBuilder::new("cube")
            .add_quad_uv([v[2], v[0], v[3], v[1]], uv(0))
            .add_quad_uv([v[7], v[5], v[6], v[4]], uv(1))
            .add_quad_uv([v[6], v[4], v[2], v[0]], uv(2))
            .add_quad_uv([v[3], v[1], v[7], v[5]], uv(3))
            .add_quad_uv([v[0], v[4], v[1], v[5]], uv(4))
            .add_quad_uv([v[6], v[2], v[7], v[3]], uv(5))
            .build()
    }

    pub fn bounding_box(bounding_box: BoundingBox) -> Arc<MeshGeometry> {
        let (left, bottom, back) = bounding_box.bottom_left().into();
        let (right, top, front) = bounding_box.top_right().into();

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

        let vi = [
            2, 3, 1, 2, 1, 0, // F
            7, 6, 4, 7, 4, 5, // B
            6, 2, 0, 6, 0, 4, // L
            3, 7, 5, 3, 5, 1, // R
            0, 1, 5, 0, 5, 4, // U
            6, 7, 3, 6, 3, 2, // D
        ];

        let mut builder = MeshGeometry::builder("bounds");

        for vertex in v {
            let vertex_data = VertexData::builder().position(vertex.into()).build();

            builder.vertex(vertex_data);
        }

        builder.generate_tangents(false).indices(&vi, false);

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

pub struct ShapeBuilder {
    builder: MeshBuilder,
    colors: Vec<Color>,
}

impl ShapeBuilder {
    pub fn new(name: &str) -> Self {
        let builder = MeshGeometry::builder(name);

        Self {
            builder,
            colors: vec![Color::WHITE],
        }
    }

    pub fn colors(mut self, colors: &[Color]) -> Self {
        self.colors.clear();
        self.colors.extend_from_slice(colors);

        self
    }

    pub fn add_vertex(
        mut self,
        position: [f32; 3],
        color: usize,
        normal: [f32; 3],
        uv: [f32; 2],
    ) -> Self {
        let vertex_data = VertexData::builder()
            .position(position.into())
            .color(self.colors[color % self.colors.len()])
            .normal(normal.into())
            .texture(uv.into())
            .build();

        self.builder.vertex(vertex_data);

        self
    }

    fn add_face(mut self, vertices: &[[f32; 3]], uv: &[[f32; 2]], indices: &[usize]) -> Self {
        debug_assert!(vertices.len() >= 3);
        debug_assert!(vertices.len() == uv.len());

        let normal = Self::normal(
            vertices[indices[0]],
            vertices[indices[1]],
            vertices[indices[2]],
        );
        for &i in indices {
            self = self.add_vertex(vertices[i], i, normal, uv[i]);
        }

        self
    }

    pub fn add_triangle(self, width: f32, height: f32) -> Self {
        let (top, bottom, left, right) = (height / 2., -height / 2., -width / 2., width / 2.);

        let v = [
            [left, bottom, 0.],
            [right, bottom, 0.],
            [(left + right) / 2., top, 0.],
        ];

        let t = [
            [T_MIN, T_MAX],
            [T_MAX, T_MAX],
            [(T_MIN + T_MAX) / 2., T_MIN],
        ];

        self.add_face(&v, &t, &[0, 1, 2])
    }

    pub fn add_quad(self, corners: [[f32; 3]; 4]) -> Self {
        self.add_quad_uv(corners, QUAD_UV)
    }

    pub fn add_quad_uv(self, corners: [[f32; 3]; 4], uv: [[f32; 2]; 4]) -> Self {
        self.add_face(&corners, &uv, &[0, 2, 3, 3, 1, 0])
    }

    fn add_hex(self, width: f32, height: f32) -> Self {
        let center = [0., 0., 0.];
        let ne = [width / 2., height, 0.];
        let e = [width, 0., 0.];
        let se = [width / 2., -height, 0.];
        let sw = [-width / 2., -height, 0.];
        let w = [-width, 0., 0.];
        let nw = [-width / 2., height, 0.];

        let v = [center, ne, e, se, sw, w, nw];

        let t = [
            [T_MID, T_MID],
            [T_MAX, T_MAX],
            [T_MAX, T_MID],
            [T_MAX, T_MIN],
            [T_MIN, T_MIN],
            [T_MIN, T_MID],
            [T_MIN, T_MAX],
        ];

        self.add_face(
            &v,
            &t,
            &[0, 2, 1, 0, 3, 2, 0, 4, 3, 0, 5, 4, 0, 6, 5, 0, 1, 6],
        )
    }

    fn normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();

        [n[0] / len, n[1] / len, n[2] / len]
    }

    pub fn build(self) -> Arc<MeshGeometry> {
        self.builder.build()
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{Color, logger, utils::timer::Timer};

    use crate::{BoundingBox, Shapes, resources::mesh::shape::ShapeBuilder};

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_shapes() {
        setup();

        let mut timer = Timer::new();
        let n = 1000;

        for _ in 0..n {
            let _ = Shapes::triangle(&[Color::RED, Color::BLUE, Color::GREEN], 1.);
        }
        tracing::info!(target: logger::RENDER, "Build {} triangles: {}", n, 1000. * timer.delta());

        for _ in 0..n {
            let _ = Shapes::rect(&[Color::RED], 1., 0., 0., 1.);
        }
        tracing::info!(target: logger::RENDER, "Build {} rects: {}", n, 1000. * timer.delta());

        let bounding_box = BoundingBox::default();
        for _ in 0..n {
            let _ = Shapes::bounding_box(bounding_box);
        }
        tracing::info!(target: logger::RENDER, "Build {} boxes: {}", n, 1000. * timer.delta());
    }

    #[test]
    fn test_normal() {
        setup();

        let size = 1.;

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

        let n1 = [
            [0., 0., 1.],
            [0., 0., -1.],
            [-1., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., -1., 0.],
        ];

        let n2 = [
            ShapeBuilder::normal(v[2], v[3], v[1]), // F
            ShapeBuilder::normal(v[7], v[6], v[4]), // B
            ShapeBuilder::normal(v[6], v[2], v[0]), // L
            ShapeBuilder::normal(v[3], v[7], v[5]), // R
            ShapeBuilder::normal(v[0], v[1], v[5]), // U
            ShapeBuilder::normal(v[6], v[7], v[3]), // D
        ];

        assert_eq!(n1, n2);
    }
}
