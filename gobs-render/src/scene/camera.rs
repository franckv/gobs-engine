use cgmath::{Deg, Matrix4, SquareMatrix, Point3, Vector3};
use cgmath::{ortho, perspective};

enum ProjectionMode {
    ORTHO,
    PERSPECTIVE
}

pub struct Camera {
    position: Point3<f32>,
    projection: Matrix4<f32>,
    view: Matrix4<f32>,
    mode: ProjectionMode,
    fov: f32,
    near: f32,
    far: f32
}

impl Camera {
    pub fn new(position: Point3<f32>) -> Camera {
        Camera {
            position: position,
            projection: Matrix4::identity(),
            view: Matrix4::identity(),
            mode: ProjectionMode::ORTHO,
            fov: 0.,
            near: 0.,
            far: 0.
        }
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }

    fn get_correction() -> Matrix4<f32> {
        // vulkan use a different coord system than opengl
        // the ortho matrix needs to be corrected
        let mut correction = Matrix4::identity();
        correction.y.y = -1.0;
        correction.z.z = 0.5;
        correction.w.z = 0.5;

        correction
    }

    pub fn set_ortho(&mut self, near: f32, far: f32) {
        self.mode = ProjectionMode::ORTHO;
        self.near = near;
        self.far = far;
    }

    pub fn set_perspective(&mut self, fov: f32, near: f32, far: f32) {
        self.mode = ProjectionMode::PERSPECTIVE;
        self.near = near;
        self.far = far;
        self.fov = fov;
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let near = self.near;
        let far = self.far;
        let fov = self.fov;
        let correction = Self::get_correction();

        match self.mode {
            ProjectionMode::ORTHO => {
                self.projection = correction * ortho(
                    -width / 2.0, width / 2.0,
                    -height / 2.0, height / 2.0,
                    near, far
                );
            },
            ProjectionMode::PERSPECTIVE => {
                let aspect = width / height;
                self.projection = correction * perspective(Deg(fov), aspect, near, far);
            }
        }
    }

    pub fn look_at(&mut self, direction: Vector3<f32>, up: Vector3<f32>) {
        self.view = Matrix4::look_at_dir(self.position.clone().into(), direction, up);
    }

    pub fn combined(&self) -> Matrix4<f32> {
        self.projection * self.view
    }
}
