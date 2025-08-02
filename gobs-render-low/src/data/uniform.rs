use std::sync::Arc;

use gobs_core::memory::allocator::Allocable;
use gobs_gfx::{Buffer, BufferUsage, GfxBuffer, GfxDevice};
use uuid::Uuid;

pub struct UniformBuffer {
    pub layout: Arc<UniformLayout>,
    pub buffer: GfxBuffer,
}

impl UniformBuffer {
    pub fn new(device: &GfxDevice, layout: Arc<UniformLayout>) -> Self {
        let buffer = GfxBuffer::new("uniform", layout.size(), BufferUsage::Uniform, device);

        UniformBuffer { layout, buffer }
    }

    pub fn update(&mut self, uniform_data: &[u8]) {
        self.buffer.copy(uniform_data, 0);
    }
}

impl Allocable<GfxDevice, Arc<UniformLayout>> for UniformBuffer {
    fn resource_id(&self) -> Uuid {
        self.buffer.id()
    }

    fn family(&self) -> Arc<UniformLayout> {
        self.layout.clone()
    }

    fn resource_size(&self) -> usize {
        1
    }

    fn allocate(device: &GfxDevice, _name: &str, _size: usize, layout: Arc<UniformLayout>) -> Self {
        Self::new(device, layout)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum UniformProp {
    Bool,
    F32,
    U32,
    U64,
    Vec2F,
    Vec3F,
    Vec4F,
    Mat3F,
    Mat4F,
}

impl UniformProp {
    fn alignment(&self) -> usize {
        match self {
            UniformProp::Bool => 4,
            UniformProp::F32 => 4,
            UniformProp::U32 => 4,
            UniformProp::U64 => 16,
            UniformProp::Vec2F => 8,
            UniformProp::Vec3F => 16,
            UniformProp::Vec4F => 16,
            UniformProp::Mat3F => 16,
            UniformProp::Mat4F => 16,
        }
    }

    fn size(&self) -> usize {
        match self {
            UniformProp::Bool => 4,
            UniformProp::F32 => 4,
            UniformProp::U32 => 4,
            UniformProp::U64 => 8,
            UniformProp::Vec2F => 8,
            UniformProp::Vec3F => 12,
            UniformProp::Vec4F => 16,
            UniformProp::Mat3F => 48,
            UniformProp::Mat4F => 64,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct UniformLayout {
    layout: Vec<UniformProp>,
    alignment: usize,
}

impl UniformLayout {
    pub fn builder() -> UniformLayoutBuilder {
        UniformLayoutBuilder::new()
    }

    pub fn len(&self) -> usize {
        self.layout.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn size(&self) -> usize {
        self.layout
            .iter()
            .map(|p| {
                let padding = (self.alignment - p.size() % self.alignment) % self.alignment;

                p.size() + padding
            })
            .sum()
    }

    pub fn data(&self, props: &[UniformPropData]) -> Vec<u8> {
        let mut data = Vec::new();

        self.copy_data(props, &mut data);

        data
    }

    pub fn copy_data(&self, props: &[UniformPropData], data: &mut Vec<u8>) {
        assert_eq!(self.len(), props.len(), "Invalid uniform layout");

        for prop in props {
            prop.copy(data);
            let pad = (self.alignment - prop.ty().size() % self.alignment) % self.alignment;
            for _ in 0..pad {
                data.push(0_u8);
            }
        }
    }
}

pub struct UniformLayoutBuilder {
    layout: Vec<UniformProp>,
}

impl UniformLayoutBuilder {
    fn new() -> Self {
        UniformLayoutBuilder { layout: Vec::new() }
    }

    pub fn prop(mut self, _label: &str, prop: UniformProp) -> Self {
        self.layout.push(prop);

        self
    }

    pub fn build(self) -> Arc<UniformLayout> {
        let alignment = self.layout.iter().map(|p| p.alignment()).max().unwrap();

        Arc::new(UniformLayout {
            layout: self.layout,
            alignment,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UniformPropData {
    Bool(bool),
    F32(f32),
    U32(u32),
    U64(u64),
    Vec2F([f32; 2]),
    Vec3F([f32; 3]),
    Vec4F([f32; 4]),
    Mat3F([[f32; 3]; 3]),
    Mat4F([[f32; 4]; 4]),
}

impl UniformPropData {
    fn ty(&self) -> UniformProp {
        match self {
            UniformPropData::Bool(_) => UniformProp::Bool,
            UniformPropData::F32(_) => UniformProp::F32,
            UniformPropData::U32(_) => UniformProp::U32,
            UniformPropData::U64(_) => UniformProp::U64,
            UniformPropData::Vec2F(_) => UniformProp::Vec2F,
            UniformPropData::Vec3F(_) => UniformProp::Vec3F,
            UniformPropData::Vec4F(_) => UniformProp::Vec4F,
            UniformPropData::Mat3F(_) => UniformProp::Mat3F,
            UniformPropData::Mat4F(_) => UniformProp::Mat4F,
        }
    }

    fn copy(&self, data: &mut Vec<u8>) {
        match self {
            UniformPropData::Bool(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            UniformPropData::F32(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            UniformPropData::U32(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            UniformPropData::U64(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            UniformPropData::Vec2F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
            UniformPropData::Vec3F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
            UniformPropData::Vec4F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
            UniformPropData::Mat3F(d) => {
                // mat3 is padded as mat3x4
                let d2 = &[
                    [d[0][0], d[0][1], d[0][2], 0.],
                    [d[1][0], d[1][1], d[1][2], 0.],
                    [d[2][0], d[2][1], d[2][2], 0.],
                ];
                data.extend_from_slice(bytemuck::cast_slice(d2))
            }
            UniformPropData::Mat4F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
        };
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use glam::{Mat4, Vec4};
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::data::uniform::{UniformLayout, UniformProp, UniformPropData};

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct CameraUniform {
        view_position: [f32; 4],
        view_proj: [[f32; 4]; 4],
    }

    #[test]
    fn test_camera() {
        let camera_data = CameraUniform {
            view_position: Vec4::ONE.into(),
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let camera_layout = UniformLayout::builder()
            .prop("position", UniformProp::Vec4F)
            .prop("view_proj", UniformProp::Mat4F)
            .build();

        let camera = camera_layout.data(&[
            UniformPropData::Vec4F(camera_data.view_position),
            UniformPropData::Mat4F(camera_data.view_proj),
        ]);

        assert_eq!(camera_layout.size(), camera.len());

        assert_eq!(camera, bytemuck::cast_slice(&[camera_data]));
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct LightUniform {
        position: [f32; 3],
        _padding: u32,
        colour: [f32; 3],
        _padding2: u32,
    }

    #[test]
    fn test_light() {
        let mut light_data = LightUniform {
            position: Vec3::ZERO.into(),
            _padding: 0,
            colour: Vec3::ONE.into(),
            _padding2: 0,
        };

        let light_layout = UniformLayout::builder()
            .prop("position", UniformProp::Vec3F)
            .prop("colour", UniformProp::Vec3F)
            .build();

        let light = light_layout.data(&[
            UniformPropData::Vec3F(light_data.position),
            UniformPropData::Vec3F(light_data.colour),
        ]);

        assert_eq!(light_layout.size(), light.len());

        assert_eq!(light, bytemuck::cast_slice(&[light_data]));

        light_data._padding = 1;
        assert_ne!(light, bytemuck::cast_slice(&[light_data]));
    }

    #[test]
    fn test_align() {
        setup();
        let mat3 = [[0 as f32; 3]; 3];
        let layout = UniformLayout::builder()
            .prop("mat3", UniformProp::Mat3F)
            .build();

        let mat = layout.data(&[UniformPropData::Mat3F(mat3)]);

        assert_eq!(mat.len(), UniformProp::Mat3F.size());
    }

    #[test]
    fn test_push() {
        setup();
        let mat3 = [[0 as f32; 3]; 3];
        let mat4 = [[0 as f32; 4]; 4];
        let u = 0_u64;

        let layout = UniformLayout::builder()
            .prop("mat4", UniformProp::Mat4F)
            .prop("mat3", UniformProp::Mat3F)
            .prop("u64", UniformProp::U64)
            .build();

        let uniform = layout.data(&[
            UniformPropData::Mat4F(mat4),
            UniformPropData::Mat3F(mat3),
            UniformPropData::U64(u),
        ]);

        assert_eq!(uniform.len(), layout.size());
        assert_eq!(uniform.len(), 128);
    }
}
