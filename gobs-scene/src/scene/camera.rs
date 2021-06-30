use cgmath::{Deg, Matrix4, SquareMatrix, Point3, Vector3};
use cgmath::{ortho, perspective};

#[derive(Copy, Clone, Debug, PartialEq)]
enum ProjectionMode {
    ORTHO,
    PERSPECTIVE
}

pub struct Camera {
    position: Point3<f32>,
    projection: Matrix4<f32>,
    view: Matrix4<f32>,
    mode: ProjectionMode,
    left: f32, top: f32, right: f32, bottom: f32,
    aspect: f32,
    fov: f32,
    near: f32,
    far: f32
}

impl Camera {
    pub fn ortho(left: f32, top: f32, right: f32, bottom: f32) -> Camera {
        let near = -10.;
        let far = 10.;

        let correction = Self::correction();

        let projection = correction * ortho(
            left, right, bottom, top,
            near, far
        );

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::ORTHO,
            left, top, right, bottom,
            aspect: 1.,
            fov: 60.,
            near,
            far
        }
    }

    pub fn perspective(near: f32, far: f32, fov: f32, aspect: f32) -> Camera {
        let correction = Self::correction();

        let projection =
            correction * perspective(Deg(fov), aspect, near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::PERSPECTIVE,
            left: -1.,
            right: 1.,
            bottom: -1.,
            top: 1.,
            aspect,
            fov,
            near,
            far
        }
    }

    pub fn set_position<P: Into<Point3<f32>>>(&mut self, position: P) {
        self.position = position.into();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;

        self.update_projection();
    }

    pub fn resize(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.left = left;
        self.right = right;
        self.bottom = bottom;
        self.top = top;

        self.update_projection();
    }

    fn update_projection(&mut self) {
        match self.mode {
            ProjectionMode::ORTHO => {
                self.projection = Self::correction() * ortho(
                    self.left, self.right, self.bottom, self.top,
                    self.near, self.far
                );
            },
            ProjectionMode::PERSPECTIVE => {
                self.projection =
                    Self::correction() * perspective(Deg(self.fov), self.aspect, self.near, self.far);
            }
        }
    }

    fn correction() -> Matrix4<f32> {
        // vulkan use a different coord system than opengl
        // the ortho matrix needs to be corrected
        let mut correction = Matrix4::identity();
        correction.y.y = -1.0;
        correction.z.z = 0.5;
        correction.w.z = 0.5;

        correction
    }

    pub fn transform(&mut self, transform: Matrix4<f32>) {
        self.view = transform * self.view
    }

    pub fn look_at<V: Into<Vector3<f32>>>(&mut self, direction: V, up: V) {
        self.view = Matrix4::look_to_rh(
            self.position.clone().into(), direction.into(), up.into());
    }

    pub fn combined(&self) -> Matrix4<f32> {
        self.projection * self.view
    }
}
