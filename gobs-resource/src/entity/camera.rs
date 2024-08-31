use core::fmt;

use glam::{Mat4, Vec3};
use uuid::Uuid;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum ProjectionMode {
    Ortho(OrthoProjection),
    Perspective(PerspectiveProjection),
}

impl ProjectionMode {
    pub fn near(&self) -> f32 {
        match self {
            ProjectionMode::Ortho(p) => p.near,
            ProjectionMode::Perspective(p) => p.near,
        }
    }

    pub fn far(&self) -> f32 {
        match self {
            ProjectionMode::Ortho(p) => p.far,
            ProjectionMode::Perspective(p) => p.far,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PerspectiveProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, Debug)]
pub struct OrthoProjection {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

pub type CameraId = Uuid;

#[derive(Clone, Debug)]
pub struct Camera {
    pub id: CameraId,
    pub mode: ProjectionMode,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn perspective(aspect: f32, fovy: f32, near: f32, far: f32, yaw: f32, pitch: f32) -> Self {
        let projection = PerspectiveProjection {
            aspect,
            fovy,
            near,
            far,
        };

        Camera {
            id: Uuid::new_v4(),
            mode: ProjectionMode::Perspective(projection),
            yaw,
            pitch,
        }
    }

    // Ortho Camera (origin top/left)
    pub fn ortho(width: f32, height: f32, near: f32, far: f32, yaw: f32, pitch: f32) -> Self {
        let projection = OrthoProjection {
            width,
            height,
            near,
            far,
        };

        Camera {
            id: Uuid::new_v4(),
            mode: ProjectionMode::Ortho(projection),
            yaw,
            pitch,
        }
    }

    pub fn view_proj(&self, position: Vec3) -> Mat4 {
        self.proj_matrix() * self.view_matrix(position)
    }

    pub fn dir(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(sin_yaw * cos_pitch, sin_pitch, -cos_pitch * cos_yaw).normalize()
    }

    pub fn right(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(cos_yaw, 0., sin_yaw).normalize()
    }

    pub fn up(&self) -> Vec3 {
        -self.dir().cross(self.right())
    }

    pub fn view_matrix(&self, position: Vec3) -> Mat4 {
        Mat4::look_to_rh(position, self.dir(), self.up())
    }

    pub fn proj_matrix(&self) -> Mat4 {
        let mut proj = match &self.mode {
            ProjectionMode::Ortho(projection) => Mat4::orthographic_rh(
                -projection.width / 2.,
                projection.width / 2.,
                -projection.height / 2.,
                projection.height / 2.,
                projection.near,
                projection.far,
            ),
            ProjectionMode::Perspective(projection) => Mat4::perspective_rh(
                projection.fovy,
                projection.aspect,
                projection.near,
                projection.far,
            ),
        };

        proj.y_axis.y *= -1.;

        proj
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        match &mut self.mode {
            ProjectionMode::Ortho(projection) => {
                projection.width = width as f32;
                projection.height = height as f32;
            }
            ProjectionMode::Perspective(projection) => {
                projection.aspect = width as f32 / height as f32;
            }
        }
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.mode {
            ProjectionMode::Ortho(projection) => {
                write!(
                    f,
                    "Yaw={}° Pitch={}° Size={}/{} dir={}",
                    self.yaw.to_degrees(),
                    self.pitch.to_degrees(),
                    projection.width,
                    projection.height,
                    self.dir(),
                )
            }
            ProjectionMode::Perspective(projection) => {
                write!(
                    f,
                    "Yaw={}° Pitch={}° Fov={}° dir={}",
                    self.yaw.to_degrees(),
                    self.pitch.to_degrees(),
                    projection.fovy.to_degrees(),
                    self.dir(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use tracing::Level;
    use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

    use crate::entity::camera::Camera;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap();
    }

    fn check_dir(yaw: f32, pitch: f32, expected: Vec3) {
        tracing::debug!("yaw={:?}, pitch={:?}, dir={:?}", yaw, pitch, expected);

        let camera = Camera::ortho(
            320. as f32,
            200. as f32,
            0.1,
            100.,
            yaw.to_radians(),
            pitch.to_radians(),
        );

        let dir = camera.dir();

        let dx = (dir.x - expected.x).abs();
        let dy = (dir.y - expected.y).abs();
        let dz = (dir.z - expected.z).abs();

        let epsilon = 0.00001;
        assert!(dx < epsilon);
        assert!(dy < epsilon);
        assert!(dz < epsilon);
    }

    #[test]
    fn test_dir() {
        setup();

        check_dir(0., 0., Vec3::new(0., 0., -1.));
        check_dir(0., -90., Vec3::new(0., -1., 0.));
        check_dir(0., 90., Vec3::new(0., 1., 0.));
        check_dir(-90., 0., Vec3::new(-1., 0., 0.));
        check_dir(-90., -90., Vec3::new(0., -1., 0.));
        check_dir(-90., 90., Vec3::new(0., 1., 0.));
        check_dir(90., 0., Vec3::new(1., 0., 0.));
        check_dir(90., 90., Vec3::new(0., 1., 0.));
        check_dir(90., -90., Vec3::new(0., -1., 0.));
        check_dir(180., 0., Vec3::new(0., 0., 1.));
        check_dir(0., 180., Vec3::new(0., 0., 1.));
        check_dir(180., 180., Vec3::new(0., 0., -1.));
    }
}
