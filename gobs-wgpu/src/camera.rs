use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};
use log::*;
use winit::event::*;
use winit::dpi::PhysicalPosition;
use wgpu::util::DeviceExt;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pub position: Vec3,
    yaw: f32,
    pitch: f32,
    pub projection: Projection,
    pub buffer: wgpu::Buffer,
    pub uniform: CameraUniform,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup
}

impl Camera {
    pub fn new<V: Into<Vec3>>(
        device: &wgpu::Device,
        position: V,
        yaw: f32,
        pitch: f32,
        width: u32, 
        height: u32,
        fovy: f32,
        znear: f32,
        zfar: f32
    ) -> Self {
        let position: Vec3 = position.into();
        let projection = Projection::new(width, height, fovy, znear, zfar);

        let uniform = CameraUniform {
            view_position: position.extend(1.0).to_array(),
            view_proj: (projection.calc_matrix() * Self::view_proj(position.into(), yaw.into(), pitch.into())).to_cols_array_2d()
        };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ],
            label: Some("camera_bind_group_layout")
        });

            
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ],
            label: Some("camera_bind_group")
        });

        Camera {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection,
            buffer,
            uniform,
            layout,
            bind_group
        }
    }

    pub fn update_view_proj(&mut self) {
        self.uniform.view_position = self.position.extend(1.0).to_array();
        self.uniform.view_proj = (self.projection.calc_matrix() * self.calc_matrix()).to_cols_array_2d();
    }

    fn calc_matrix(&self) -> Mat4 {
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

pub struct Projection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32
}

impl Projection {
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

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4]
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
        info!("Process input {:?}", key);

        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };

        match key {
            VirtualKeyCode::Z | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::Q | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward = Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += self.rotate_vertical * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}