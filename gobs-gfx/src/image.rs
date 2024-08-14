use crate::{ImageExtent2D, ImageFormat, ImageUsage, SamplerFilter};

pub trait Image {
    type GfxDevice;

    fn new(
        name: &str,
        device: &Self::GfxDevice,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self;
    fn invalidate(&mut self);
    fn extent(&self) -> ImageExtent2D;
}

pub trait Sampler {
    type GfxDevice;

    fn new(device: &Self::GfxDevice, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self;
}
