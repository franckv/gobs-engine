use serde::Serialize;

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize)]
pub enum AlignMode {
    Compact,
    #[default]
    Scalar,
    Std140,
    Std430,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Attribute {
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

impl Attribute {
    pub fn size(self) -> usize {
        match self {
            Attribute::Bool => 1,
            Attribute::F32 => 4,
            Attribute::U32 => 4,
            Attribute::U64 => 8,
            Attribute::Vec2F => 8,
            Attribute::Vec3F => 12,
            Attribute::Vec4F => 16,
            Attribute::Mat3F => 48,
            Attribute::Mat4F => 64,
        }
    }

    pub fn alignment(self, mode: AlignMode) -> usize {
        match mode {
            AlignMode::Compact => 1,
            AlignMode::Scalar => match self {
                Attribute::Bool => 4,
                Attribute::F32 => 4,
                Attribute::U32 => 4,
                Attribute::U64 => 8,
                Attribute::Vec2F => 4,
                Attribute::Vec3F => 4,
                Attribute::Vec4F => 4,
                Attribute::Mat3F => 4,
                Attribute::Mat4F => 4,
            },
            AlignMode::Std140 | AlignMode::Std430 => match self {
                Attribute::Bool => 4,
                Attribute::F32 => 4,
                Attribute::U32 => 4,
                Attribute::U64 => 8,
                Attribute::Vec2F => 8,
                Attribute::Vec3F => 16,
                Attribute::Vec4F => 16,
                Attribute::Mat3F => 16,
                Attribute::Mat4F => 16,
            },
        }
    }

    /// Round offset to a multiple of align
    pub fn round_up(offset: usize, align: usize) -> usize {
        offset.div_ceil(align) * align
    }

    pub fn offsets(attributes: &[Attribute], mode: AlignMode) -> Vec<usize> {
        let mut total_offset = 0;

        attributes
            .iter()
            .map(|a| {
                total_offset = Self::round_up(total_offset, a.alignment(mode));
                let offset = total_offset;
                total_offset += a.size();

                offset
            })
            .collect()
    }

    pub fn stride(attributes: &[Attribute], mode: AlignMode) -> usize {
        let mut stride = 0;
        let mut max_align = 1;

        for attr in attributes {
            let align = attr.alignment(mode);
            max_align = align.max(max_align);
            stride = Self::round_up(stride, align);
            stride += attr.size();
        }

        match mode {
            AlignMode::Std140 => Self::round_up(stride, 16),
            _ => Self::round_up(stride, max_align),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum AttributeData {
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

impl AttributeData {
    pub fn ty(&self) -> Attribute {
        match self {
            AttributeData::Bool(_) => Attribute::Bool,
            AttributeData::F32(_) => Attribute::F32,
            AttributeData::U32(_) => Attribute::U32,
            AttributeData::U64(_) => Attribute::U64,
            AttributeData::Vec2F(_) => Attribute::Vec2F,
            AttributeData::Vec3F(_) => Attribute::Vec3F,
            AttributeData::Vec4F(_) => Attribute::Vec4F,
            AttributeData::Mat3F(_) => Attribute::Mat3F,
            AttributeData::Mat4F(_) => Attribute::Mat4F,
        }
    }

    pub fn copy(&self, data: &mut Vec<u8>) {
        match self {
            AttributeData::Bool(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            AttributeData::F32(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            AttributeData::U32(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            AttributeData::U64(d) => data.extend_from_slice(bytemuck::cast_slice(&[*d])),
            AttributeData::Vec2F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
            AttributeData::Vec3F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
            AttributeData::Vec4F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
            AttributeData::Mat3F(d) => {
                // mat3 is padded as mat3x4
                let d2 = &[
                    [d[0][0], d[0][1], d[0][2], 0.],
                    [d[1][0], d[1][1], d[1][2], 0.],
                    [d[2][0], d[2][1], d[2][2], 0.],
                ];
                data.extend_from_slice(bytemuck::cast_slice(d2))
            }
            AttributeData::Mat4F(d) => data.extend_from_slice(bytemuck::cast_slice(d)),
        };
    }

    pub fn pad(data: &mut Vec<u8>, len: usize) {
        data.resize(data.len() + len, 0);
    }

    pub fn offsets(attributes: &[AttributeData], mode: AlignMode) -> Vec<usize> {
        let mut total_offset = 0;

        attributes
            .iter()
            .map(|data| {
                let a = data.ty();
                total_offset = Attribute::round_up(total_offset, a.alignment(mode));
                let offset = total_offset;
                total_offset += a.size();

                offset
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::data::align::{AlignMode, Attribute};

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_offsets() {
        setup();

        let layout = &[Attribute::Vec3F, Attribute::Vec4F];
        let offsets = Attribute::offsets(layout, AlignMode::Compact);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 12);

        let layout = &[Attribute::Vec3F, Attribute::Vec4F, Attribute::Vec3F];
        let offsets = Attribute::offsets(layout, AlignMode::Compact);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 12);
        assert_eq!(offsets[2], 28);
    }

    #[test]
    fn test_round_up() {
        setup();

        assert_eq!(Attribute::round_up(0, 4), 0);
        assert_eq!(Attribute::round_up(1, 4), 4);
        assert_eq!(Attribute::round_up(2, 4), 4);
        assert_eq!(Attribute::round_up(3, 4), 4);
        assert_eq!(Attribute::round_up(4, 4), 4);

        assert_eq!(Attribute::round_up(0, 1), 0);
        assert_eq!(Attribute::round_up(1, 1), 1);
        assert_eq!(Attribute::round_up(2, 1), 2);
        assert_eq!(Attribute::round_up(3, 1), 3);
        assert_eq!(Attribute::round_up(4, 1), 4);

        assert_eq!(Attribute::round_up(8, 4), 8);
        assert_eq!(Attribute::round_up(16, 4), 16);
        assert_eq!(Attribute::round_up(10, 4), 12);
        assert_eq!(Attribute::round_up(10, 16), 16);
        assert_eq!(Attribute::round_up(17, 16), 32);
    }

    #[test]
    fn test_stride() {
        setup();

        let layout = &[Attribute::F32];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 4);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 4);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 16);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 4);

        let layout = &[Attribute::U64];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 8);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 8);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 16);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 8);

        let layout = &[Attribute::Vec2F, Attribute::Vec2F];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 16);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 16);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 16);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 16);

        let layout = &[Attribute::Vec3F, Attribute::Vec2F];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 20);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 20);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 32);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 32);

        let layout = &[Attribute::Vec3F, Attribute::Vec4F];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 28);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 28);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 32);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 32);

        let layout = &[Attribute::Vec3F, Attribute::Vec4F, Attribute::Vec3F];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 40);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 40);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 48);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 48);

        let layout = &[Attribute::Mat3F, Attribute::Mat4F];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 112);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 112);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 112);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 112);

        let layout = &[Attribute::F32, Attribute::Mat3F];
        assert_eq!(Attribute::stride(layout, AlignMode::Compact), 52);
        assert_eq!(Attribute::stride(layout, AlignMode::Scalar), 52);
        assert_eq!(Attribute::stride(layout, AlignMode::Std140), 64);
        assert_eq!(Attribute::stride(layout, AlignMode::Std430), 64);
    }
}
