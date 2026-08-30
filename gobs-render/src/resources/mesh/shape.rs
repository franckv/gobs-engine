use std::sync::Arc;

use glam::Vec2;

use gobs_core::Color;
use gobs_render_hal::VertexData;

use crate::{MeshBuilder, resources::MeshGeometry};

pub struct Shapes;

impl Shapes {
    pub fn triangle(colors: &[Color], size: f32) -> Arc<MeshGeometry> {
        ShapeBuilder::new("triangle")
            .with_colors(colors)
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
            .with_colors(colors)
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
            .with_colors(colors)
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
            .with_colors(colors)
            .add_quad([v[2], v[0], v[3], v[1]])
            .add_quad([v[7], v[5], v[6], v[4]])
            .add_quad([v[6], v[4], v[2], v[0]])
            .add_quad([v[3], v[1], v[7], v[5]])
            .add_quad([v[0], v[4], v[1], v[5]])
            .add_quad([v[6], v[2], v[7], v[3]])
            .build()
    }

    pub fn cubemap(cols: usize, rows: usize, index: &[usize], size: f32) -> Arc<MeshGeometry> {
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
            .with_atlas(cols, rows)
            .with_atlas_index(index)
            .add_quad([v[2], v[0], v[3], v[1]])
            .add_quad([v[7], v[5], v[6], v[4]])
            .add_quad([v[6], v[4], v[2], v[0]])
            .add_quad([v[3], v[1], v[7], v[5]])
            .add_quad([v[0], v[4], v[1], v[5]])
            .add_quad([v[6], v[2], v[7], v[3]])
            .build()
    }
}

pub struct ShapeBuilder {
    builder: MeshBuilder,
    default_colors: Vec<Color>,
    colors: Option<Vec<Color>>,
    normals: Option<Vec<[f32; 3]>>,
    atlas: Option<(usize, usize)>,
    atlas_index: Option<Vec<usize>>,
    face_index: usize,
    geometry_only: bool,
}

impl ShapeBuilder {
    pub fn new(name: &str) -> Self {
        let builder = MeshGeometry::builder(name);

        Self {
            builder,
            default_colors: vec![Color::WHITE],
            colors: None,
            normals: None,
            atlas: None,
            atlas_index: None,
            face_index: 0,
            geometry_only: false,
        }
    }

    pub fn with_colors(mut self, colors: &[Color]) -> Self {
        if !colors.is_empty() {
            self.colors = Some(colors.to_vec());
        }

        self
    }

    pub fn with_normals(mut self, normals: &[[f32; 3]]) -> Self {
        self.normals = Some(normals.to_vec());

        self
    }

    pub fn with_atlas(mut self, cols: usize, rows: usize) -> Self {
        self.atlas = Some((cols, rows));

        self
    }

    pub fn with_atlas_index(mut self, index: &[usize]) -> Self {
        self.atlas_index = Some(index.to_vec());

        self
    }

    pub fn geometry_only(mut self) -> Self {
        self.geometry_only = true;
        self.builder.generate_tangents(false);

        self
    }

    fn get_atlas_index(&self) -> usize {
        if let Some(index) = &self.atlas_index {
            index[self.face_index % index.len()]
        } else {
            self.face_index
        }
    }

    fn get_colors(&self) -> &[Color] {
        if let Some(colors) = &self.colors {
            colors
        } else {
            &self.default_colors
        }
    }

    fn default_uv(corners: usize) -> Vec<[f32; 2]> {
        let t_min: f32 = 0.01;
        let t_mid: f32 = 0.5;
        let t_max: f32 = 1. - t_min;

        if corners == 3 {
            vec![
                [t_min, t_max],
                [t_max, t_max],
                [(t_min + t_max) / 2., t_min],
            ]
        } else if corners == 4 {
            vec![
                [t_min, t_max],
                [t_min, t_min],
                [t_max, t_max],
                [t_max, t_min],
            ]
        } else if corners == 7 {
            vec![
                [t_mid, t_mid],
                [t_max, t_max],
                [t_max, t_mid],
                [t_max, t_min],
                [t_min, t_min],
                [t_min, t_mid],
                [t_min, t_max],
            ]
        } else {
            todo!()
        }
    }

    fn tex_map(tex_coords: Vec2, cols: usize, rows: usize, index: usize) -> Vec2 {
        let col = (index % cols) as f32;
        let row = (index / cols) as f32;

        let u = (col + tex_coords.x) / cols as f32;
        let v = (row + tex_coords.y) / rows as f32;

        Vec2::new(u, v)
    }

    fn get_uv(&self, corners: usize) -> Vec<[f32; 2]> {
        if let Some((cols, rows)) = self.atlas {
            Self::default_uv(corners)
                .into_iter()
                .map(|c| Self::tex_map(c.into(), cols, rows, self.get_atlas_index()).into())
                .collect::<Vec<_>>()
        } else {
            Self::default_uv(corners)
        }
    }

    pub fn add_vertices(mut self, vertices: &[VertexData], indices: &[u32]) -> Self {
        self.builder.indices(indices, true).vertices(vertices);

        self
    }

    pub fn add_vertex(
        mut self,
        position: [f32; 3],
        color: usize,
        normal: [f32; 3],
        uv: [f32; 2],
    ) -> Self {
        let colors = self.get_colors();

        let vertex_data = VertexData::builder()
            .position(position.into())
            .color(colors[color % colors.len()])
            .normal(normal.into())
            .texture(uv.into())
            .build();

        self.builder.vertex(vertex_data);

        self
    }

    fn add_face(mut self, vertices: &[[f32; 3]], indices: &[usize]) -> Self {
        debug_assert!(vertices.len() >= 3);
        debug_assert!(
            self.normals
                .as_ref()
                .is_none_or(|n| { n.len() == 1 || n.len() == vertices.len() })
        );

        if self.geometry_only {
            for &i in indices {
                self = self.add_vertex(vertices[i], 0, [0.; 3], [0.; 2]);
            }
        } else {
            let face_normal = Self::normal(
                vertices[indices[0]],
                vertices[indices[1]],
                vertices[indices[2]],
            );

            let uv = self.get_uv(vertices.len());

            for &i in indices {
                let normal = if let Some(normals) = &self.normals {
                    normals[i % normals.len()]
                } else {
                    face_normal
                };
                let uv = uv[i % uv.len()];

                self = self.add_vertex(vertices[i], i, normal, uv);
            }
        }

        self.face_index += 1;

        self
    }

    pub fn add_triangle(self, width: f32, height: f32) -> Self {
        let (top, bottom, left, right) = (height / 2., -height / 2., -width / 2., width / 2.);

        let v = [
            [left, bottom, 0.],
            [right, bottom, 0.],
            [(left + right) / 2., top, 0.],
        ];

        self.add_face(&v, &[0, 1, 2])
    }

    pub fn add_quad(self, corners: [[f32; 3]; 4]) -> Self {
        self.add_face(&corners, &[0, 2, 3, 3, 1, 0])
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

        self.add_face(&v, &[0, 2, 1, 0, 3, 2, 0, 4, 3, 0, 5, 4, 0, 6, 5, 0, 1, 6])
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

    use crate::{Shapes, resources::mesh::shape::ShapeBuilder};

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
