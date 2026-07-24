use crate::{
    BindResource, BindingGroupLayout, BufferType, RenderHAL,
    data::{AlignMode, Attribute, align::AttributeData},
};

pub struct UniformBuffer {
    pub buffer: BindResource,
}

impl UniformBuffer {
    pub fn new(
        label: &str,
        hal: &mut dyn RenderHAL,
        bind_layout: BindingGroupLayout,
        data_layout: &UniformLayout,
    ) -> Self {
        let buffer = hal.create_buffer(label, data_layout.size(), BufferType::Uniform);

        UniformBuffer {
            buffer: BindResource::new(bind_layout, vec![buffer]),
        }
    }

    pub fn update(&self, hal: &mut dyn RenderHAL, uniform_data: &[u8]) {
        hal.upload_buffer(self.buffer.slot(0).unwrap(), uniform_data, 0);
    }
}

pub trait UniformData<DataProp> {
    fn prop(self, prop: DataProp) -> Self
    where
        Self: Sized;

    fn uniform_layout(&self) -> &UniformLayout;

    fn copy_data<F>(&self, buffer: &mut Vec<u8>, get_data: F)
    where
        F: Fn(&DataProp) -> AttributeData;

    fn is_empty(&self) -> bool;
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UniformLayout {
    layout: Vec<Attribute>,
    mode: AlignMode,
}

impl UniformLayout {
    pub fn new(mode: AlignMode) -> Self {
        Self {
            layout: Vec::new(),
            mode,
        }
    }

    pub fn prop(mut self, _label: &str, prop: Attribute) -> Self {
        self.layout.push(prop);

        self
    }

    pub fn len(&self) -> usize {
        self.layout.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn size(&self) -> usize {
        Attribute::stride(&self.layout, self.mode)
    }

    pub fn data(&self, props: &[AttributeData]) -> Vec<u8> {
        let mut data = Vec::new();

        self.copy_data(props, &mut data);

        data
    }

    pub fn copy_data(&self, props: &[AttributeData], data: &mut Vec<u8>) {
        debug_assert_eq!(self.len(), props.len(), "Invalid uniform layout");
        debug_assert!(
            self.layout
                .iter()
                .zip(props)
                .all(|(attr, data)| data.ty() == *attr)
        );
        let offsets = AttributeData::offsets(props, self.mode);

        let data_start = data.len();
        for (prop, offset) in props.iter().zip(offsets) {
            let position = data.len() - data_start;
            AttributeData::pad(data, offset - position);
            prop.copy(data);
        }
        let position = data.len() - data_start;
        AttributeData::pad(data, self.size() - position);
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use glam::{Mat4, Vec4};
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::data::align::AttributeData;
    use crate::data::uniform::UniformLayout;
    use crate::data::{AlignMode, Attribute};

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

        let camera_layout = UniformLayout::new(AlignMode::Std140)
            .prop("position", Attribute::Vec4F)
            .prop("view_proj", Attribute::Mat4F);

        let camera = camera_layout.data(&[
            AttributeData::Vec4F(camera_data.view_position),
            AttributeData::Mat4F(camera_data.view_proj),
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

        let light_layout = UniformLayout::new(AlignMode::Std140)
            .prop("position", Attribute::Vec3F)
            .prop("colour", Attribute::Vec3F);

        let light = light_layout.data(&[
            AttributeData::Vec3F(light_data.position),
            AttributeData::Vec3F(light_data.colour),
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
        let layout = UniformLayout::new(AlignMode::Std140).prop("mat3", Attribute::Mat3F);

        let mat = layout.data(&[AttributeData::Mat3F(mat3)]);

        assert_eq!(mat.len(), Attribute::Mat3F.size());
    }

    #[test]
    fn test_push() {
        setup();
        let mat3 = [[0 as f32; 3]; 3];
        let mat4 = [[0 as f32; 4]; 4];
        let u = 0_u64;

        let layout = UniformLayout::new(AlignMode::Std430)
            .prop("mat4", Attribute::Mat4F)
            .prop("mat3", Attribute::Mat3F)
            .prop("u64", Attribute::U64);

        let uniform = layout.data(&[
            AttributeData::Mat4F(mat4),
            AttributeData::Mat3F(mat3),
            AttributeData::U64(u),
        ]);

        assert_eq!(uniform.len(), layout.size());
        assert_eq!(uniform.len(), 128);
    }
}
