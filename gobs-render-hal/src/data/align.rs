use serde::Serialize;

#[allow(unused)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
pub enum AlignMode {
    Compact,
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
