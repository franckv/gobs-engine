use cgmath::{Deg, Matrix4, SquareMatrix, Point3, Vector3};
use cgmath::{ortho, perspective};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProjectionMode {
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
    width: f32,
    height: f32,
    aspect: f32,
    fov: f32,
    near: f32,
    far: f32
}

impl Camera {
    pub fn ortho(width: f32, height: f32) -> Self {
        let near = -10.;
        let far = 10.;

        let projection = Self::ortho_projection(width, height, near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::Ortho,
            width,
            height,
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

        let projection = Self::ortho_projection(width, height, near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::OrthoFixedWidth,
            width, 
            height,
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

        let projection = Self::ortho_projection(width, height, near, far);

        Camera {
            position: [0., 0., 0.].into(),
            projection,
            view: Matrix4::identity(),
            mode: ProjectionMode::OrthoFixedHeight,
            width, 
            height,
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
            width: 1.,
            height: 1.,
            aspect,
            fov,
            near,
            far
        }
    }

    pub fn set_mode(&mut self, mode: ProjectionMode) {
        self.mode = mode;
    }

    pub fn set_position<P: Into<Point3<f32>>>(&mut self, position: P) {
        self.position = position.into();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;

        self.update();
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.update();
    }

    fn perspective_projection(fov: f32, aspect: f32, near: f32, far: f32) -> Matrix4<f32> {
        Self::correction() * perspective(Deg(fov), aspect, near, far)
    }

    fn ortho_projection(width: f32, height: f32, near: f32, far: f32) -> Matrix4<f32> {
        let left = -width / 2.;
        let right = width / 2.;
        let top = height / 2.;
        let bottom = -height / 2.;

        Self::correction() * ortho(
            left, right, bottom, top, near, far
        )
    }

    fn update(&mut self) {
        match self.mode {
            ProjectionMode::Ortho => {
                self.projection = Self::ortho_projection(self.width, self.height, self.near, self.far);
            },
            ProjectionMode::Perspective => {
                self.projection = Self::perspective_projection(self.fov, self.aspect, self.near, self.far);
            },
            ProjectionMode::OrthoFixedHeight => {
                self.width = self.height * self.aspect;

                self.projection = Self::ortho_projection(self.width, self.height, self.near, self.far);

            },
            ProjectionMode::OrthoFixedWidth => {
                self.height = self.width / self.aspect;

                self.projection = Self::ortho_projection(self.width, self.height, self.near, self.far);

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
