use glam::{Mat4, Vec3};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ProjectionMode {
    Ortho,
    Perspective,
    OrthoFixedWidth,
    OrthoFixedHeight,
}

#[derive(Debug)]
pub struct CameraProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32
}

impl CameraProjection {
    pub fn new(
        width: u32, 
        height: u32,
        fovy: f32,
        znear: f32,
        zfar: f32
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    #[allow(dead_code)]
    mode: ProjectionMode,
    pub yaw: f32,
    pub pitch: f32,
    pub projection: CameraProjection,
}

impl Camera {
    pub fn new<V: Into<Vec3>>(
        position: V,
        projection: CameraProjection,
        yaw: f32,
        pitch: f32
    ) -> Self {
        Camera {
            position: position.into(),
            mode: ProjectionMode::Perspective,
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Self::view_proj(self.position, self.yaw, self.pitch)
    }

    fn view_proj(
        position: Vec3,
        yaw: f32,
        pitch: f32,
    ) -> Mat4 {
        let (sin_pitch, cos_pitch) = pitch.sin_cos();
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        let dir = Vec3::new(
            cos_pitch * cos_yaw,
            sin_pitch,
            cos_pitch * sin_yaw
        ).normalize();

        Mat4::look_to_rh(
            position,
            dir,
            Vec3::Y
        )
    }
}