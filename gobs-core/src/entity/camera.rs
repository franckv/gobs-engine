use core::fmt;

use glam::{Mat4, Vec3};
use uuid::Uuid;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct PerspectiveProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug)]
pub struct OrthoProjection {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

pub type CameraId = Uuid;

#[derive(Debug)]
pub struct Camera {
    pub id: CameraId,
    pub position: Vec3,
    pub mode: ProjectionMode,
    pub yaw: f32,
    pub pitch: f32,
    pub up: Vec3,
}

impl Camera {
    pub fn perspective<V: Into<Vec3>>(
        position: V,
        aspect: f32,
        fovy: f32,
        near: f32,
        far: f32,
        yaw: f32,
        pitch: f32,
        up: Vec3,
    ) -> Self {
        let projection = PerspectiveProjection {
            aspect,
            fovy,
            near,
            far,
        };

        Camera {
            id: Uuid::new_v4(),
            position: position.into(),
            mode: ProjectionMode::Perspective(projection),
            yaw,
            pitch,
            up,
        }
    }

    // Ortho Camera (origin top/left)
    pub fn ortho<V: Into<Vec3>>(
        position: V,
        width: f32,
        height: f32,
        near: f32,
        far: f32,
        yaw: f32,
        pitch: f32,
        up: Vec3,
    ) -> Self {
        let projection = OrthoProjection {
            width,
            height,
            near,
            far,
        };

        Camera {
            id: Uuid::new_v4(),
            position: position.into(),
            mode: ProjectionMode::Ortho(projection),
            yaw,
            pitch,
            up,
        }
    }

    pub fn view_proj(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    pub fn dir(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        let dir = self.dir();

        Mat4::look_to_rh(self.position, dir, self.up)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        match &self.mode {
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
        }
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
                    "Position={} Yaw={}° Pitch={}° Size={}/{} dir={}",
                    self.position,
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
                    "Position={} Yaw={}° Pitch={}° Fov={}° dir={}",
                    self.position,
                    self.yaw.to_degrees(),
                    self.pitch.to_degrees(),
                    projection.fovy.to_degrees(),
                    self.dir(),
                )
            }
        }
    }
}
