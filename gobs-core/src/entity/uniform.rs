#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UniformProp {
    F32,
    U32,
    U64,
    Vec2F,
    Vec3F,
    Vec4F,
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
            UniformProp::Mat4F => 64,
        }
    }
}

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

    pub fn prop(mut self, prop: UniformProp) -> Self {
        self.layout.push(prop);

        self
    }

    pub fn build(self) -> UniformLayout {
        UniformLayout {
            layout: self.layout,
        }
    }
}

#[derive(Clone, Copy)]
pub enum UniformPropData {
    F32(f32),
    U32(u32),
    U64(u64),
    Vec2F([f32; 2]),
    Vec3F([f32; 3]),
    Vec4F([f32; 4]),
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

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct CameraUniform {
        view_position: [f32; 4],
        view_proj: [[f32; 4]; 4],
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct _LightUniform {
        position: [f32; 3],
        _padding: u32,
        colour: [f32; 3],
        _padding2: u32,
    }

    #[test]
    fn test_raw() {
        let camera_data = CameraUniform {
            view_position: Vec4::ONE.into(),
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let camera_layout = UniformLayout::builder()
            .prop(UniformProp::Vec4F)
            .prop(UniformProp::Mat4F)
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

        let mut light_data = _LightUniform {
            position: Vec3::ZERO.into(),
            _padding: 0,
            colour: Vec3::ONE.into(),
            _padding2: 0,
        };

        let light_layout = UniformLayout::builder()
            .prop(UniformProp::Vec3F)
            .prop(UniformProp::Vec3F)
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
}
