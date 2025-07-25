use core::fmt;

use glam::{Mat4, Vec3, Vec4};
use uuid::Uuid;

use gobs_core::Transform;

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

impl Default for Camera {
    fn default() -> Self {
        Self {
            id: Default::default(),
            mode: ProjectionMode::Ortho(OrthoProjection {
                width: 800.,
                height: 600.,
                near: 0.,
                far: 10.,
            }),
            yaw: 0.,
            pitch: 0.,
        }
    }
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

    pub fn screen_to_ndc(&self, pos: Vec3, width: f32, height: f32) -> Vec3 {
        Vec3::new(2. * pos.x / width - 1., 2. * pos.y / height - 1., pos.z)
    }

    pub fn screen_to_world(
        &self,
        pos: Vec3,
        camera_transform: Transform,
        width: f32,
        height: f32,
    ) -> Vec4 {
        let view_proj = self.view_proj(camera_transform.translation());
        let view_proj_inv = view_proj.inverse();

        let pos_ndc = self.screen_to_ndc(pos, width, height).extend(1.);

        view_proj_inv * pos_ndc
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
    use tracing::{Level, level_filters::LevelFilter};
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::logger;

    use crate::entity::camera::Camera;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    fn check_dir(yaw: f32, pitch: f32, expected: Vec3) {
        tracing::debug!(target: logger::RESOURCES, "yaw={:?}, pitch={:?}, dir={:?}", yaw, pitch, expected);

        let camera = Camera::ortho(
            320_f32,
            200_f32,
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
