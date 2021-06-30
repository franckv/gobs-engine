use cgmath::{Deg, Matrix4, SquareMatrix, Point3, Vector3};
use cgmath::{ortho, perspective};

#[derive(Copy, Clone, Debug, PartialEq)]
enum ProjectionMode {
    Ortho,
    Perspective,
    OrthoFixedWidth,
    OrthoFixedHeight
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
    pub fn ortho(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        let near = -10.;
        let far = 10.;

        let projection = Self::ortho_projection(left, top, right, bottom,
            near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::Ortho,
            left, top, right, bottom,
            aspect: 1.,
            fov: 60.,
            near,
            far
        }
    }

    pub fn ortho_fixed_width(width: f32, aspect: f32) -> Self {
        let near = -10.;
        let far = 10.;
        let fov = 60.;

        let height = width / aspect;

        let left = -width / 2.;
        let right = width / 2.;
        let top = height / 2.;
        let bottom = -height / 2.;

        let projection = Self::ortho_projection(left, top, right, bottom,
            near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::OrthoFixedWidth,
            left, top, right, bottom,
            aspect,
            fov,
            near,
            far
        }
    }

    pub fn ortho_fixed_height(height: f32, aspect: f32) -> Self {
        let near = -10.;
        let far = 10.;
        let fov = 60.;

        let width = height * aspect;

        let left = -width / 2.;
        let right = width / 2.;
        let top = height / 2.;
        let bottom = -height / 2.;

        let projection = Self::ortho_projection(left, top, right, bottom,
            near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::OrthoFixedHeight,
            left, top, right, bottom,
            aspect,
            fov,
            near,
            far
        }
    }

    pub fn perspective(near: f32, far: f32, fov: f32, aspect: f32) -> Self {
        let correction = Self::correction();

        let projection =
            correction * perspective(Deg(fov), aspect, near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::Perspective,
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

        self.update();
    }

    pub fn resize(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.left = left;
        self.right = right;
        self.bottom = bottom;
        self.top = top;

        self.update();
    }

    fn perspective_projection(fov: f32, aspect: f32, near: f32, far: f32) -> Matrix4<f32> {
        Self::correction() * perspective(Deg(fov), aspect, near, far)
    }

    fn ortho_projection(left: f32, top: f32, right: f32, bottom: f32, near: f32, far: f32) -> Matrix4<f32> {
        Self::correction() * ortho(
            left, right, bottom, top, near, far
        )
    }

    fn update(&mut self) {
        match self.mode {
            ProjectionMode::Ortho => {
                self.projection = Self::ortho_projection(self.left, self.top, self.right, self.bottom,
                    self.near, self.far);
            },
            ProjectionMode::Perspective => {
                self.projection = Self::perspective_projection(self.fov, self.aspect, self.near, self.far);
            },
            ProjectionMode::OrthoFixedHeight => {
                let height = (self.top - self.bottom).abs();
                let width = height * self.aspect;

                self.left = -width / 2.;
                self.right = width / 2.;

                self.projection = Self::ortho_projection(self.left, self.top, self.right, self.bottom,
                    self.near, self.far);

            },
            ProjectionMode::OrthoFixedWidth => {
                let width = (self.right - self.left).abs();
                let height = width / self.aspect;

                self.top = height / 2.;
                self.bottom = -height / 2.;

                self.projection = Self::ortho_projection(self.left, self.top, self.right, self.bottom,
                    self.near, self.far);

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
