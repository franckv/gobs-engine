use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UniformProp {
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

#[derive(Debug)]
pub struct UniformLayout {
    layout: Vec<UniformProp>,
}

impl UniformLayout {
    pub fn builder() -> UniformLayoutBuilder {
        UniformLayoutBuilder::new()
    }

    pub fn len(&self) -> usize {
        self.layout.len()
    }

    fn alignment(&self) -> usize {
        let alignment = self.layout.iter().map(|p| p.alignment()).max();

        alignment.unwrap()
    }

    pub fn size(&self) -> usize {
        let alignment = self.alignment();

        self.layout
            .iter()
            .map(|p| {
                let padding = (alignment - p.size() % alignment) % alignment;

                p.size() + padding
            })
            .sum()
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
        Arc::new(UniformLayout {
            layout: self.layout,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UniformPropData {
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

    fn raw(&self) -> Vec<u8> {
        match self {
            UniformPropData::F32(d) => bytemuck::cast_slice(&[*d]).into(),
            UniformPropData::U32(d) => bytemuck::cast_slice(&[*d]).into(),
            UniformPropData::U64(d) => bytemuck::cast_slice(&[*d]).into(),
            UniformPropData::Vec2F(d) => bytemuck::cast_slice(d).into(),
            UniformPropData::Vec3F(d) => bytemuck::cast_slice(d).into(),
            UniformPropData::Vec4F(d) => bytemuck::cast_slice(d).into(),
            UniformPropData::Mat3F(d) => {
                // mat3 is padded as mat3x4
                let d2 = &[
                    [d[0][0], d[0][1], d[0][2], 0.],
                    [d[1][0], d[1][1], d[1][2], 0.],
                    [d[2][0], d[2][1], d[2][2], 0.],
                ];
                bytemuck::cast_slice(d2).into()
            }
            UniformPropData::Mat4F(d) => bytemuck::cast_slice(d).into(),
        }
    }
}

#[derive(Clone)]
pub struct UniformData {
    pub data: Vec<UniformPropData>,
}

impl UniformData {
    pub fn new(layout: &UniformLayout, props: &[UniformPropData]) -> Self {
        assert_eq!(layout.len(), props.len(), "Invalid uniform layout");

        let mut data: Vec<UniformPropData> = Vec::new();

        for (&prop, &prop_data) in layout.layout.iter().zip(props.iter()) {
            assert_eq!(prop, prop_data.ty(), "Invalid uniform layout");

            data.push(prop_data);
        }

        UniformData { data }
    }

    pub fn raw(&self) -> Vec<u8> {
        let alignment = self.alignment();

        self.data
            .iter()
            .flat_map(|p| {
                let mut raw = p.raw();

                let align = (alignment - raw.len() % alignment) % alignment;

                for _ in 0..align {
                    raw.push(0 as u8);
                }

                raw
            })
            .collect::<Vec<u8>>()
    }

    fn alignment(&self) -> usize {
        let alignment = self.data.iter().map(|p| p.ty().alignment()).max();

        alignment.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use glam::{Mat4, Vec4};

    use crate::entity::uniform::{UniformData, UniformLayout, UniformProp, UniformPropData};

    fn setup() {
        let _ = env_logger::Builder::new()
            .filter_module("gobs_core", log::LevelFilter::Debug)
            .try_init();
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

        let camera = UniformData::new(
            &camera_layout,
            &[
                UniformPropData::Vec4F(camera_data.view_position),
                UniformPropData::Mat4F(camera_data.view_proj),
            ],
        );

        assert_eq!(camera_layout.size(), camera.raw().len());

        assert_eq!(camera.raw(), bytemuck::cast_slice(&[camera_data]));
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

        let light = UniformData::new(
            &light_layout,
            &[
                UniformPropData::Vec3F(light_data.position),
                UniformPropData::Vec3F(light_data.colour),
            ],
        );

        assert_eq!(light_layout.size(), light.raw().len());

        assert_eq!(light.raw(), bytemuck::cast_slice(&[light_data]));

        light_data._padding = 1;
        assert_ne!(light.raw(), bytemuck::cast_slice(&[light_data]));
    }

    #[test]
    fn test_align() {
        setup();

        let mat3 = [[0 as f32; 3]; 3];
        let layout = UniformLayout::builder()
            .prop("mat3", UniformProp::Mat3F)
            .build();

        let data = UniformData::new(&layout, &[UniformPropData::Mat3F(mat3)]);

        let raw = data.raw();

        assert_eq!(raw.len(), UniformProp::Mat3F.size());
    }

    #[test]
    fn test_push() {
        setup();

        let mat3 = [[0 as f32; 3]; 3];
        let mat4 = [[0 as f32; 4]; 4];
        let u = 0 as u64;

        let layout = UniformLayout::builder()
            .prop("mat4", UniformProp::Mat4F)
            .prop("mat3", UniformProp::Mat3F)
            .prop("u64", UniformProp::U64)
            .build();

        let data = UniformData::new(
            &layout,
            &[
                UniformPropData::Mat4F(mat4),
                UniformPropData::Mat3F(mat3),
                UniformPropData::U64(u),
            ],
        );

        let raw = data.raw();

        assert_eq!(raw.len(), layout.size());
        assert_eq!(raw.len(), 128);
    }
}
