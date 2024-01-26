use indexmap::IndexMap;

pub enum UniformProp {}

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
    pub fn alignment(&self) -> u32 {
        match self {
            UniformPropData::F32(_) => 4,
            UniformPropData::U32(_) => 4,
            UniformPropData::U64(_) => 16,
            UniformPropData::Vec2F(_) => 8,
            UniformPropData::Vec3F(_) => 16,
            UniformPropData::Vec4F(_) => 16,
            UniformPropData::Mat4F(_) => 16,
        }
    }

    pub fn raw(&self) -> Vec<u8> {
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
    pub name: String,
    pub data: IndexMap<String, UniformPropData>,
}

impl UniformData {
    pub fn builder(name: &str) -> UniformDataBuilder {
        UniformDataBuilder::new(name)
    }

    pub fn prop(&self, name: &str) -> UniformPropData {
        self.data[name]
    }

    pub fn update(&mut self, name: &str, prop: UniformPropData) {
        self.data[name] = prop;
    }

    pub fn raw(&self) -> Vec<u8> {
        let alignment = self.alignment() as usize;

        self.data
            .values()
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

    pub fn alignment(&self) -> u32 {
        let alignment = self.data.values().map(|p| p.alignment()).max();

        alignment.unwrap()
    }
}

pub struct UniformDataBuilder {
    pub name: String,
    pub data: IndexMap<String, UniformPropData>,
}

impl UniformDataBuilder {
    pub fn new(name: &str) -> Self {
        UniformDataBuilder {
            name: name.to_string(),
            data: IndexMap::new(),
        }
    }

    pub fn prop(mut self, name: &str, prop: UniformPropData) -> Self {
        self.data.insert(name.to_string(), prop);

        self
    }

    pub fn build(self) -> UniformData {
        UniformData {
            name: self.name,
            data: self.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use glam::{Mat4, Vec4};

    use super::UniformDataBuilder;
    use super::UniformPropData;

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

        let camera = UniformDataBuilder::new("camera")
            .prop(
                "view_position",
                UniformPropData::Vec4F(camera_data.view_position),
            )
            .prop("view_proj", UniformPropData::Mat4F(camera_data.view_proj))
            .build();

        assert_eq!(camera.raw(), bytemuck::cast_slice(&[camera_data]));

        let mut light_data = _LightUniform {
            position: Vec3::ZERO.into(),
            _padding: 0,
            colour: Vec3::ONE.into(),
            _padding2: 0,
        };

        let light = UniformDataBuilder::new("light")
            .prop("position", UniformPropData::Vec3F(light_data.position))
            .prop("colour", UniformPropData::Vec3F(light_data.colour))
            .build();

        assert_eq!(light.raw(), bytemuck::cast_slice(&[light_data]));

        light_data._padding = 1;
        assert_ne!(light.raw(), bytemuck::cast_slice(&[light_data]));
    }
}
