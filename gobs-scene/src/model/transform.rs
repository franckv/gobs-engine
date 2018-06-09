use cgmath::{Deg, Matrix, Matrix3, Matrix4, SquareMatrix, Vector3, Vector4};

#[derive(Clone)]
pub struct Transform {
    matrix: Matrix4<f32>
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            matrix: Matrix4::identity()
        }
    }

    pub fn translation<V: Into<Vector3<f32>>>(v: V) -> Self {
        Transform {
            matrix: Matrix4::from_translation(v.into())
        }
    }

    pub fn scaling(scale_x: f32, scale_y: f32, scale_z: f32) -> Self {
        Transform {
            matrix: Matrix4::from_diagonal(Vector4::new(scale_x, scale_y, scale_z, 1.))
        }
    }

    pub fn rotation<V: Into<Vector3<f32>>>(axis: V, angle: f32) -> Self {
        Transform {
            matrix: Matrix4::from_axis_angle(axis.into(), Deg(angle))
        }
    }

    pub fn translate<V: Into<Vector3<f32>>>(self, v: V) -> Self {
        self.transform(&Transform::translation(v))
    }

    pub fn transform(mut self, t: &Transform) -> Self {
        self.matrix = t.matrix * self.matrix;

        self
    }

    pub fn normal_transform(&self) -> Transform {
        let mat: Matrix3<f32> = self.clone().into();
        Transform {
            matrix: mat.invert().unwrap().transpose().into()
        }
    }
}

impl From<Transform> for Matrix3<f32> {
    fn from(t: Transform) -> Matrix3<f32> {
        Matrix3::from_cols(
            t.matrix.x.truncate(),
            t.matrix.y.truncate(),
            t.matrix.z.truncate()
        )
    }
}

impl From<Transform> for [[f32; 3]; 3] {
    fn from(t: Transform) -> [[f32; 3]; 3] {
        Matrix3::from_cols(
            t.matrix.x.truncate(),
            t.matrix.y.truncate(),
            t.matrix.z.truncate()
        ).into()
    }
}

impl From<Transform> for [[f32; 4]; 4] {
    fn from(t: Transform) -> [[f32; 4]; 4] {
        t.matrix.into()
    }
}
